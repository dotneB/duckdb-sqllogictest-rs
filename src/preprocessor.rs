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

pub fn preprocess_file(path: &Path) -> Result<Option<PreprocessRun>> {
    let script = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read: {}", path.display()))?;

    let mut directives = PreprocessDirectives::default();
    let mut out = String::with_capacity(script.len());
    let mut did_change = false;

    for raw in script.split_inclusive('\n') {
        let (line, eol) = if let Some(stripped) = raw.strip_suffix("\r\n") {
            (stripped, "\r\n")
        } else if let Some(stripped) = raw.strip_suffix('\n') {
            (stripped, "\n")
        } else {
            (raw, "")
        };

        let trimmed = line.trim_start();
        if trimmed.starts_with('#') {
            out.push_str(line);
            out.push_str(eol);
            continue;
        }

        let Some((head, rest)) = trimmed.split_once(char::is_whitespace) else {
            out.push_str(line);
            out.push_str(eol);
            continue;
        };

        match head {
            // DuckDB compatibility: `require <name>` / `require '<name>'`
            "require" => {
                did_change = true;
                if let Some(ext) = parse_single_token_name(rest) {
                    directives.required_extensions.push(ext);
                }
                // Make upstream sqllogictest parser ignore the directive without shifting lines.
                out.push_str("# ");
                out.push_str(trimmed);
                out.push_str(eol);
            }
            _ => {
                out.push_str(line);
                out.push_str(eol);
            }
        }
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
