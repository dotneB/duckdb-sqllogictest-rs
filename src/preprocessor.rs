use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

#[derive(Default, Debug, Clone)]
pub struct PreprocessDirectives {
    pub required_extensions: Vec<String>,
}

const UNSUPPORTED_REQUIRED_DIRECTIVES: &[&str] = &["mode"];

#[derive(Debug)]
pub struct PreprocessRun {
    preprocessed: PathBuf,
    pub directives: PreprocessDirectives,
}

impl PreprocessRun {
    pub fn preprocessed_path(&self) -> &Path {
        &self.preprocessed
    }
}

impl Drop for PreprocessRun {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.preprocessed);
    }
}

trait DirectiveHandler {
    fn handle_line(&self, line: &str) -> Option<HandlerOutput>;
}

#[derive(Debug, PartialEq, Eq)]
struct HandlerOutput {
    rewritten_line: String,
    required_extension: Option<String>,
    skip_next_record: bool,
}

struct RequireDirectiveHandler;

impl DirectiveHandler for RequireDirectiveHandler {
    fn handle_line(&self, line: &str) -> Option<HandlerOutput> {
        let trimmed = line.trim_start();
        let (head, rest) = trimmed.split_once(char::is_whitespace)?;
        if head != "require" {
            return None;
        }

        Some(HandlerOutput {
            rewritten_line: format!("# {trimmed}"),
            required_extension: parse_single_token_name(rest),
            skip_next_record: false,
        })
    }
}

struct SkipIfDirectiveHandler;

impl DirectiveHandler for SkipIfDirectiveHandler {
    fn handle_line(&self, line: &str) -> Option<HandlerOutput> {
        let trimmed = line.trim_start();
        let (head, rest) = trimmed.split_once(char::is_whitespace)?;
        if head != "skipif" {
            return None;
        }

        let target = parse_single_token_name(rest);
        Some(HandlerOutput {
            rewritten_line: format!("# {trimmed}"),
            required_extension: None,
            skip_next_record: target.as_deref() == Some("duckdb"),
        })
    }
}

struct OnlyIfDirectiveHandler;

impl DirectiveHandler for OnlyIfDirectiveHandler {
    fn handle_line(&self, line: &str) -> Option<HandlerOutput> {
        let trimmed = line.trim_start();
        let (head, rest) = trimmed.split_once(char::is_whitespace)?;
        if head != "onlyif" {
            return None;
        }

        let target = parse_single_token_name(rest);
        Some(HandlerOutput {
            rewritten_line: format!("# {trimmed}"),
            required_extension: None,
            skip_next_record: target.as_deref() != Some("duckdb"),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SkipState {
    None,
    AwaitingRecordStart,
    SkippingRecord,
}

pub fn preprocess_file(path: &Path) -> Result<Option<PreprocessRun>> {
    let script = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read: {}", path.display()))?;

    let mut directives = PreprocessDirectives::default();
    let mut out = String::with_capacity(script.len());
    let mut did_change = false;

    let require_handler = RequireDirectiveHandler;
    let skipif_handler = SkipIfDirectiveHandler;
    let onlyif_handler = OnlyIfDirectiveHandler;
    let handlers: [&dyn DirectiveHandler; 3] = [&require_handler, &skipif_handler, &onlyif_handler];
    let mut skip_state = SkipState::None;
    let mut line_no = 0usize;

    for raw in script.split_inclusive('\n') {
        line_no += 1;
        let (line, eol) = split_line_with_eol(raw);

        let trimmed = line.trim_start();
        if trimmed.starts_with('#') {
            out.push_str(line);
            out.push_str(eol);
            continue;
        }

        if let Some(keyword) = find_unsupported_required_directive(line) {
            return Err(anyhow::anyhow!(
                "unsupported required directive '{keyword}' at {}:{line_no}",
                path.display()
            ));
        }

        if skip_state == SkipState::AwaitingRecordStart {
            if trimmed.is_empty() {
                out.push_str(line);
                out.push_str(eol);
                continue;
            }

            did_change = true;
            out.push_str(&comment_out(line));
            out.push_str(eol);
            skip_state = SkipState::SkippingRecord;
            continue;
        }

        if skip_state == SkipState::SkippingRecord {
            if trimmed.is_empty() {
                out.push_str(line);
                out.push_str(eol);
                skip_state = SkipState::None;
                continue;
            }

            did_change = true;
            out.push_str(&comment_out(line));
            out.push_str(eol);
            continue;
        }

        if let Some(output) = run_directive_pipeline(line, &mut directives, &handlers) {
            did_change = true;
            out.push_str(&output.rewritten_line);
            out.push_str(eol);
            if output.skip_next_record {
                skip_state = SkipState::AwaitingRecordStart;
            }
            continue;
        }

        out.push_str(line);
        out.push_str(eol);
    }

    if !did_change {
        return Ok(None);
    }

    let preprocessed = write_preprocessed_temp_file(path, &out)?;
    Ok(Some(PreprocessRun {
        preprocessed,
        directives,
    }))
}

fn split_line_with_eol(raw: &str) -> (&str, &str) {
    if let Some(stripped) = raw.strip_suffix("\r\n") {
        (stripped, "\r\n")
    } else if let Some(stripped) = raw.strip_suffix('\n') {
        (stripped, "\n")
    } else {
        (raw, "")
    }
}

fn run_directive_pipeline(
    line: &str,
    directives: &mut PreprocessDirectives,
    handlers: &[&dyn DirectiveHandler],
) -> Option<HandlerOutput> {
    for handler in handlers {
        let Some(output) = handler.handle_line(line) else {
            continue;
        };

        if let Some(ext) = output.required_extension.as_ref() {
            directives.required_extensions.push(ext.clone());
        }

        return Some(output);
    }

    None
}

fn comment_out(line: &str) -> String {
    format!("# {}", line.trim_start())
}

fn find_unsupported_required_directive(line: &str) -> Option<&'static str> {
    let token = line.split_whitespace().next()?;
    UNSUPPORTED_REQUIRED_DIRECTIVES
        .iter()
        .copied()
        .find(|keyword| *keyword == token)
}

fn parse_single_token_name(rest: &str) -> Option<String> {
    let token = rest.split_whitespace().next()?.trim();
    if token.is_empty() {
        return None;
    }

    let name = token.strip_prefix('\'').unwrap_or(token);
    let name = name.strip_suffix('\'').unwrap_or(name);
    let name = name.trim();
    if name.is_empty() {
        return None;
    }

    Some(name.to_string())
}

fn write_preprocessed_temp_file(original: &Path, script: &str) -> Result<PathBuf> {
    let parent = original.parent().unwrap_or_else(|| Path::new("."));
    let file_name = original
        .file_name()
        .map(|s| s.to_string_lossy())
        .unwrap_or_else(|| "test.slt".into());

    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let tmp_name = format!(
        "{}.duckdb-slt-preprocessed-{}-{}",
        file_name,
        std::process::id(),
        nanos
    );
    let tmp_path = parent.join(tmp_name);

    std::fs::write(&tmp_path, script)
        .with_context(|| format!("failed to write: {}", tmp_path.display()))?;
    Ok(tmp_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_temp_script_path() -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "duckdb-slt-preprocessor-test-{}-{}.slt",
            std::process::id(),
            nanos
        ))
    }

    fn run_preprocessor(script: &str) -> (PathBuf, Option<PreprocessRun>) {
        let path = make_temp_script_path();
        std::fs::write(&path, script).unwrap();
        let run = preprocess_file(&path).unwrap();
        (path, run)
    }

    fn read_preprocessed(run: &PreprocessRun) -> String {
        std::fs::read_to_string(run.preprocessed_path()).unwrap()
    }

    fn cleanup_input(path: &Path) {
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn require_handler_matches_and_rewrites_basic_require() {
        let handler = RequireDirectiveHandler;
        let output = handler.handle_line("require httpfs").unwrap();

        assert_eq!(output.rewritten_line, "# require httpfs");
        assert_eq!(output.required_extension.as_deref(), Some("httpfs"));
        assert!(!output.skip_next_record);
    }

    #[test]
    fn require_handler_rewrite_drops_leading_whitespace() {
        let handler = RequireDirectiveHandler;
        let output = handler.handle_line("   require httpfs").unwrap();

        assert_eq!(output.rewritten_line, "# require httpfs");
        assert_eq!(output.required_extension.as_deref(), Some("httpfs"));
        assert!(!output.skip_next_record);
    }

    #[test]
    fn require_handler_parses_quoted_extension_token() {
        let handler = RequireDirectiveHandler;
        let output = handler.handle_line("require 'httpfs'").unwrap();

        assert_eq!(output.rewritten_line, "# require 'httpfs'");
        assert_eq!(output.required_extension.as_deref(), Some("httpfs"));
        assert!(!output.skip_next_record);
    }

    #[test]
    fn require_handler_parses_token_with_single_leading_quote() {
        let handler = RequireDirectiveHandler;
        let output = handler.handle_line("require 'httpfs").unwrap();

        assert_eq!(output.rewritten_line, "# require 'httpfs");
        assert_eq!(output.required_extension.as_deref(), Some("httpfs"));
        assert!(!output.skip_next_record);
    }

    #[test]
    fn require_handler_rewrites_even_without_extension_token() {
        let handler = RequireDirectiveHandler;
        let output = handler.handle_line("require   ").unwrap();

        assert_eq!(output.rewritten_line, "# require   ");
        assert!(output.required_extension.is_none());
        assert!(!output.skip_next_record);
    }

    #[test]
    fn require_handler_does_not_match_non_require_lines() {
        let handler = RequireDirectiveHandler;

        assert!(handler.handle_line("require").is_none());
        assert!(handler.handle_line("load httpfs").is_none());
    }

    #[test]
    fn preprocess_file_preserves_line_count_and_content_mapping() {
        let script = "statement ok\nSELECT 1;\r\nrequire httpfs\r\n# require ignored\nquery I\nSELECT 2;\n----\n2\n";
        let expected = "statement ok\nSELECT 1;\r\n# require httpfs\r\n# require ignored\nquery I\nSELECT 2;\n----\n2\n";

        let (input_path, run) = run_preprocessor(script);
        let run = run.expect("require directive should trigger preprocessing");
        let output = read_preprocessed(&run);

        assert_eq!(output, expected);
        assert_eq!(
            script.split_inclusive('\n').count(),
            output.split_inclusive('\n').count()
        );
        assert_eq!(run.directives.required_extensions, vec!["httpfs"]);

        drop(run);
        cleanup_input(&input_path);
    }

    #[test]
    fn preprocess_file_preserves_newline_style_per_line() {
        let script = "require httpfs\r\nrequire parquet\nstatement ok\r\nSELECT 1;\n";

        let (input_path, run) = run_preprocessor(script);
        let run = run.expect("require directives should trigger preprocessing");
        let output = read_preprocessed(&run);

        assert_eq!(
            output,
            "# require httpfs\r\n# require parquet\nstatement ok\r\nSELECT 1;\n"
        );
        assert_eq!(
            run.directives.required_extensions,
            vec!["httpfs", "parquet"]
        );

        drop(run);
        cleanup_input(&input_path);
    }

    #[test]
    fn preprocess_file_skips_commented_require_and_returns_none() {
        let script = "# require httpfs\nstatement ok\nSELECT 1;\n";

        let (input_path, run) = run_preprocessor(script);

        assert!(run.is_none());

        cleanup_input(&input_path);
    }

    #[test]
    fn preprocess_file_skipif_duckdb_skips_following_record() {
        let script = "skipif duckdb\nquery I\nSELECT 1;\n----\n1\n\nquery I\nSELECT 2;\n----\n2\n";
        let expected =
            "# skipif duckdb\n# query I\n# SELECT 1;\n# ----\n# 1\n\nquery I\nSELECT 2;\n----\n2\n";

        let (input_path, run) = run_preprocessor(script);
        let run = run.expect("skipif duckdb should trigger preprocessing");
        let output = read_preprocessed(&run);

        assert_eq!(output, expected);
        assert_eq!(
            script.split_inclusive('\n').count(),
            output.split_inclusive('\n').count()
        );

        drop(run);
        cleanup_input(&input_path);
    }

    #[test]
    fn preprocess_file_onlyif_non_duckdb_skips_following_record() {
        let script = "onlyif sqlite\nstatement ok\nSELECT 1;\n\nstatement ok\nSELECT 2;\n";
        let expected = "# onlyif sqlite\n# statement ok\n# SELECT 1;\n\nstatement ok\nSELECT 2;\n";

        let (input_path, run) = run_preprocessor(script);
        let run = run.expect("onlyif sqlite should trigger preprocessing");
        let output = read_preprocessed(&run);

        assert_eq!(output, expected);
        assert_eq!(
            script.split_inclusive('\n').count(),
            output.split_inclusive('\n').count()
        );

        drop(run);
        cleanup_input(&input_path);
    }

    #[test]
    fn preprocess_file_onlyif_duckdb_rewrites_but_keeps_record() {
        let script = "onlyif duckdb\nstatement ok\nSELECT 1;\n";
        let expected = "# onlyif duckdb\nstatement ok\nSELECT 1;\n";

        let (input_path, run) = run_preprocessor(script);
        let run = run.expect("onlyif duckdb should trigger preprocessing");
        let output = read_preprocessed(&run);

        assert_eq!(output, expected);

        drop(run);
        cleanup_input(&input_path);
    }

    #[test]
    fn preprocess_file_reports_unsupported_required_directive_with_location() {
        let script = "mode output_hash\nstatement ok\nSELECT 1;\n";

        let path = make_temp_script_path();
        std::fs::write(&path, script).unwrap();

        let err = preprocess_file(&path).unwrap_err().to_string();
        assert!(err.contains("unsupported required directive 'mode'"));
        assert!(err.contains(&format!("{}:1", path.display())));

        cleanup_input(&path);
    }
}
