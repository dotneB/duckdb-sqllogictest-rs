use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

#[derive(Default, Debug, Clone)]
pub struct PreprocessDirectives {
    pub required_extensions: Vec<String>,
}

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
        })
    }
}

pub fn preprocess_file(path: &Path) -> Result<Option<PreprocessRun>> {
    let script = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read: {}", path.display()))?;

    let mut directives = PreprocessDirectives::default();
    let mut out = String::with_capacity(script.len());
    let mut did_change = false;
    let require_handler = RequireDirectiveHandler;
    let handlers: [&dyn DirectiveHandler; 1] = [&require_handler];

    for raw in script.split_inclusive('\n') {
        let (line, eol) = split_line_with_eol(raw);

        let trimmed = line.trim_start();
        if trimmed.starts_with('#') {
            out.push_str(line);
            out.push_str(eol);
            continue;
        }

        if let Some(rewritten_line) = run_directive_pipeline(line, &mut directives, &handlers) {
            did_change = true;
            out.push_str(&rewritten_line);
            out.push_str(eol);
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
) -> Option<String> {
    for handler in handlers {
        let Some(output) = handler.handle_line(line) else {
            continue;
        };

        if let Some(ext) = output.required_extension {
            directives.required_extensions.push(ext);
        }

        return Some(output.rewritten_line);
    }

    None
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
    }

    #[test]
    fn require_handler_rewrite_drops_leading_whitespace() {
        let handler = RequireDirectiveHandler;
        let output = handler.handle_line("   require httpfs").unwrap();

        assert_eq!(output.rewritten_line, "# require httpfs");
        assert_eq!(output.required_extension.as_deref(), Some("httpfs"));
    }

    #[test]
    fn require_handler_parses_quoted_extension_token() {
        let handler = RequireDirectiveHandler;
        let output = handler.handle_line("require 'httpfs'").unwrap();

        assert_eq!(output.rewritten_line, "# require 'httpfs'");
        assert_eq!(output.required_extension.as_deref(), Some("httpfs"));
    }

    #[test]
    fn require_handler_parses_token_with_single_leading_quote() {
        let handler = RequireDirectiveHandler;
        let output = handler.handle_line("require 'httpfs").unwrap();

        assert_eq!(output.rewritten_line, "# require 'httpfs");
        assert_eq!(output.required_extension.as_deref(), Some("httpfs"));
    }

    #[test]
    fn require_handler_rewrites_even_without_extension_token() {
        let handler = RequireDirectiveHandler;
        let output = handler.handle_line("require   ").unwrap();

        assert_eq!(output.rewritten_line, "# require   ");
        assert!(output.required_extension.is_none());
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
}
