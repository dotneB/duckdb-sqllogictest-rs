use std::borrow::Cow;

use anyhow::{Result, anyhow};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExtensionSpec<'a> {
    Name(Cow<'a, str>),
    RepositoryName {
        name: Cow<'a, str>,
        repository: Cow<'a, str>,
    },
    Path(Cow<'a, str>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtensionActions {
    pub display: String,
    pub install_sql: String,
    pub load_sql: String,
}

pub fn compile_extension_actions(raw: &str) -> Result<ExtensionActions> {
    let spec = parse_extension_spec(raw)?;
    let (install_sql, load_sql) = sql_for_extension_spec(&spec);

    Ok(ExtensionActions {
        display: raw.to_string(),
        install_sql,
        load_sql,
    })
}

pub fn parse_extension_spec(raw: &str) -> Result<ExtensionSpec<'_>> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Err(anyhow!("extension spec is empty"));
    }

    if let Some((name, repo)) = raw.rsplit_once('@') {
        if name.is_empty() || repo.is_empty() {
            return Err(anyhow!("invalid extension spec: '{raw}'"));
        }
        if name.contains('@') {
            return Err(anyhow!("invalid extension spec: '{raw}'"));
        }

        return Ok(ExtensionSpec::RepositoryName {
            name: Cow::Borrowed(name),
            repository: Cow::Borrowed(repo),
        });
    }

    if is_path_spec(raw) {
        return Ok(ExtensionSpec::Path(Cow::Borrowed(raw)));
    }

    Ok(ExtensionSpec::Name(Cow::Borrowed(raw)))
}

pub fn sql_for_extension_spec(spec: &ExtensionSpec<'_>) -> (String, String) {
    match spec {
        ExtensionSpec::Name(name) => (
            format!("INSTALL '{}';", escape_sql_string_literal(name)),
            format!("LOAD '{}';", escape_sql_string_literal(name)),
        ),
        ExtensionSpec::RepositoryName { name, repository } => (
            format!(
                "INSTALL '{}' FROM {};",
                escape_sql_string_literal(name),
                format_repository_expression(repository)
            ),
            format!("LOAD '{}';", escape_sql_string_literal(name)),
        ),
        ExtensionSpec::Path(path) => (
            format!("INSTALL '{}';", escape_sql_string_literal(path)),
            format!("LOAD '{}';", escape_sql_string_literal(path)),
        ),
    }
}

pub fn format_repository_expression(repository: &str) -> String {
    if is_sql_identifier(repository) {
        repository.to_string()
    } else {
        format!("'{}'", escape_sql_string_literal(repository))
    }
}

fn is_sql_identifier(s: &str) -> bool {
    let mut chars = s.chars();
    let Some(first) = chars.next() else {
        return false;
    };

    if !(first.is_ascii_alphabetic() || first == '_') {
        return false;
    }

    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

pub fn is_path_spec(raw: &str) -> bool {
    raw.ends_with(".duckdb_extension") || raw.contains('/') || raw.contains('\\')
}

pub fn escape_sql_string_literal(s: &str) -> String {
    s.replace('\'', "''")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_name() {
        assert_eq!(
            parse_extension_spec("json").unwrap(),
            ExtensionSpec::Name("json".into())
        );
    }

    #[test]
    fn parse_community() {
        assert_eq!(
            parse_extension_spec("spatial@community").unwrap(),
            ExtensionSpec::RepositoryName {
                name: "spatial".into(),
                repository: "community".into()
            }
        );
    }

    #[test]
    fn parse_custom_repository_url() {
        assert_eq!(
            parse_extension_spec("custom_extension@https://my-custom-extension-repository")
                .unwrap(),
            ExtensionSpec::RepositoryName {
                name: "custom_extension".into(),
                repository: "https://my-custom-extension-repository".into()
            }
        );
    }

    #[test]
    fn parse_path() {
        assert_eq!(
            parse_extension_spec("path/to/ext.duckdb_extension").unwrap(),
            ExtensionSpec::Path("path/to/ext.duckdb_extension".into())
        );
    }

    #[test]
    fn reject_malformed_spec() {
        assert!(parse_extension_spec("").is_err());
        assert!(parse_extension_spec("@community").is_err());
        assert!(parse_extension_spec("spatial@").is_err());
        assert!(parse_extension_spec("a@b@community").is_err());
    }

    #[test]
    fn sql_for_name() {
        let (install, load) = sql_for_extension_spec(&ExtensionSpec::Name("json".into()));
        assert_eq!(install, "INSTALL 'json';");
        assert_eq!(load, "LOAD 'json';");
    }

    #[test]
    fn sql_for_community() {
        let (install, load) = sql_for_extension_spec(&ExtensionSpec::RepositoryName {
            name: "spatial".into(),
            repository: "community".into(),
        });
        assert_eq!(install, "INSTALL 'spatial' FROM community;");
        assert_eq!(load, "LOAD 'spatial';");
    }

    #[test]
    fn sql_for_custom_repository_url() {
        let (install, load) = sql_for_extension_spec(&ExtensionSpec::RepositoryName {
            name: "custom_extension".into(),
            repository: "https://my-custom-extension-repository".into(),
        });
        assert_eq!(
            install,
            "INSTALL 'custom_extension' FROM 'https://my-custom-extension-repository';"
        );
        assert_eq!(load, "LOAD 'custom_extension';");
    }

    #[test]
    fn sql_for_path() {
        let (install, load) =
            sql_for_extension_spec(&ExtensionSpec::Path("path/to/ext.duckdb_extension".into()));
        assert_eq!(install, "INSTALL 'path/to/ext.duckdb_extension';");
        assert_eq!(load, "LOAD 'path/to/ext.duckdb_extension';");
    }

    #[test]
    fn escape_single_quotes_in_path() {
        let (install, load) = sql_for_extension_spec(&ExtensionSpec::Path(
            "path/with'quote/ext.duckdb_extension".into(),
        ));
        assert_eq!(install, "INSTALL 'path/with''quote/ext.duckdb_extension';");
        assert_eq!(load, "LOAD 'path/with''quote/ext.duckdb_extension';");
    }
}
