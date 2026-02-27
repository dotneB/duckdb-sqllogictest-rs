use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

pub(crate) fn expand_files(files: &[PathBuf]) -> Result<Vec<PathBuf>> {
    let mut out: Vec<PathBuf> = Vec::new();

    for path in files {
        if looks_like_glob_pattern(path) {
            let pattern = normalize_glob_pattern(path);
            let mut matches: Vec<PathBuf> = glob::glob(&pattern)
                .with_context(|| format!("invalid glob pattern: {pattern}"))?
                .map(|res| res.with_context(|| format!("failed to expand glob: {pattern}")))
                .collect::<Result<Vec<_>>>()?;

            matches.sort_by(|a, b| a.to_string_lossy().cmp(&b.to_string_lossy()));

            if matches.is_empty() {
                anyhow::bail!("glob pattern matched no files: {pattern}");
            }

            out.extend(matches);
        } else {
            out.push(path.clone());
        }
    }

    Ok(out)
}

pub(crate) fn normalize_path(path: &Path) -> Result<PathBuf> {
    let path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    };

    Ok(path)
}

pub(crate) fn format_user_path(base_dir: &Path, path: &Path) -> String {
    path.strip_prefix(base_dir)
        .unwrap_or(path)
        .display()
        .to_string()
}

pub(crate) fn format_user_path_str(base_dir: &Path, raw: &str) -> String {
    let path = Path::new(raw);
    if path.is_absolute() {
        format_user_path(base_dir, path)
    } else {
        raw.replace(['/', '\\'], std::path::MAIN_SEPARATOR_STR)
    }
}

fn looks_like_glob_pattern(path: &Path) -> bool {
    let s = path.to_string_lossy();
    s.contains('*')
        || s.contains('?')
        || s.contains('[')
        || s.contains(']')
        || s.contains('{')
        || s.contains('}')
}

fn normalize_glob_pattern(path: &Path) -> String {
    let s = path.to_string_lossy();
    if cfg!(windows) {
        s.replace('\\', "/")
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{format_user_path, format_user_path_str};

    #[test]
    fn strips_base_dir_from_display_path() {
        let base_dir = Path::new("workspace");
        let path = base_dir.join("tests").join("pass.slt");

        assert_eq!(
            format_user_path(base_dir, &path),
            Path::new("tests").join("pass.slt").display().to_string()
        );
    }

    #[test]
    fn absolute_raw_path_uses_strip_prefix_rules() {
        let base_dir = std::env::temp_dir().join("duckdb-slt-pathing-tests");
        let path = base_dir.join("fixtures").join("pass.slt");
        let raw = path.to_string_lossy();

        assert_eq!(
            format_user_path_str(&base_dir, raw.as_ref()),
            Path::new("fixtures").join("pass.slt").display().to_string()
        );
    }

    #[test]
    fn relative_raw_path_normalizes_slashes() {
        let raw = "fixtures/pass\\pass.slt";
        let expected = format!(
            "fixtures{sep}pass{sep}pass.slt",
            sep = std::path::MAIN_SEPARATOR
        );

        assert_eq!(format_user_path_str(Path::new("."), raw), expected);
    }
}
