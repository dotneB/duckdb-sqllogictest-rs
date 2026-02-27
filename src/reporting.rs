use std::collections::HashMap;
use std::path::Path;

use sqllogictest::runner::TestErrorKind;
use sqllogictest::{QueryExpect, Record};

use crate::pathing::format_user_path_str;

pub(crate) fn format_ok(use_color: bool) -> &'static str {
    if use_color { "\x1b[32mok\x1b[0m" } else { "ok" }
}

pub(crate) fn format_failed(use_color: bool) -> &'static str {
    if use_color {
        "\x1b[31mFAILED\x1b[0m"
    } else {
        "FAILED"
    }
}

pub(crate) fn format_error(use_color: bool) -> &'static str {
    if use_color {
        "\x1b[31mERROR\x1b[0m"
    } else {
        "ERROR"
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RecordId {
    index_1_based: usize,
    name: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct RecordMetadataCache {
    by_file_and_line: HashMap<String, HashMap<u32, RecordId>>,
}

impl RecordMetadataCache {
    pub(crate) fn build(main_file: &Path) -> Option<Self> {
        let records =
            sqllogictest::parse_file::<sqllogictest::DefaultColumnType>(main_file).ok()?;

        let mut cache = Self::default();
        let mut index = 0usize;

        for record in records {
            match record {
                Record::Statement {
                    loc: record_loc, ..
                }
                | Record::System {
                    loc: record_loc, ..
                } => {
                    index += 1;
                    cache.insert(&record_loc, index, None);
                }
                Record::Query {
                    loc: record_loc,
                    expected,
                    ..
                } => {
                    index += 1;
                    let name = match expected {
                        QueryExpect::Results { label, .. } => label,
                        QueryExpect::Error(_) => None,
                    };
                    cache.insert(&record_loc, index, name);
                }
                _ => {}
            }
        }

        Some(cache)
    }

    fn insert(&mut self, loc: &sqllogictest::Location, index_1_based: usize, name: Option<String>) {
        self.by_file_and_line
            .entry(loc.file().to_string())
            .or_default()
            .insert(
                loc.line(),
                RecordId {
                    index_1_based,
                    name,
                },
            );
    }

    fn find(&self, loc: &sqllogictest::Location) -> Option<RecordId> {
        self.by_file_and_line
            .get(loc.file())
            .and_then(|entries| entries.get(&loc.line()))
            .cloned()
    }
}

pub(crate) fn render_failure_report(
    display_main_file: &Path,
    parse_main_file: &Path,
    base_dir: &Path,
    test_err: &sqllogictest::TestError,
    record_metadata: Option<&RecordMetadataCache>,
) -> String {
    use std::fmt::Write;

    let kind = test_err.kind();
    let loc = test_err.location();
    let record_id = record_metadata.and_then(|metadata| metadata.find(&loc));

    let parse_main_file_str = parse_main_file.to_string_lossy();
    let display_loc_file = if loc.file() == parse_main_file_str {
        display_main_file.to_string_lossy().to_string()
    } else {
        loc.file().to_string()
    };

    let mut out = String::new();
    writeln!(
        out,
        "  at: {}:{}",
        format_user_path_str(base_dir, &display_loc_file),
        loc.line()
    )
    .expect("writing to String should not fail");
    if let Some(id) = &record_id {
        writeln!(
            out,
            "  record: {}{}",
            id.index_1_based,
            id.name
                .as_deref()
                .map(|n| format!(" name={n}"))
                .unwrap_or_default()
        )
        .expect("writing to String should not fail");
    }

    let sql = match &kind {
        TestErrorKind::Ok { sql, .. }
        | TestErrorKind::Fail { sql, .. }
        | TestErrorKind::ErrorMismatch { sql, .. }
        | TestErrorKind::StatementResultMismatch { sql, .. }
        | TestErrorKind::QueryResultMismatch { sql, .. }
        | TestErrorKind::QueryResultColumnsMismatch { sql, .. } => Some(sql.as_str()),
        TestErrorKind::ParseError(_)
        | TestErrorKind::SystemFail { .. }
        | TestErrorKind::SystemStdoutMismatch { .. } => None,
        _ => None,
    };

    if let Some(sql) = sql {
        writeln!(out, "sql:\n{sql}").expect("writing to String should not fail");
    }

    match &kind {
        TestErrorKind::QueryResultMismatch {
            expected, actual, ..
        } => {
            writeln!(out, "expected: {expected}").expect("writing to String should not fail");
            writeln!(out, "actual: {actual}").expect("writing to String should not fail");
        }
        TestErrorKind::QueryResultColumnsMismatch {
            expected, actual, ..
        } => {
            let expected_count = expected.chars().count();
            let actual_count = actual.chars().count();
            writeln!(
                out,
                "details: Expected {expected_count} columns, but got {actual_count} columns"
            )
            .expect("writing to String should not fail");
            writeln!(out, "expected_columns: {expected}")
                .expect("writing to String should not fail");
            writeln!(out, "actual_columns: {actual}").expect("writing to String should not fail");
        }
        TestErrorKind::ErrorMismatch {
            expected_err,
            err,
            actual_sqlstate,
            ..
        } => {
            writeln!(out, "expected_error: {expected_err}")
                .expect("writing to String should not fail");
            if let Some(sqlstate) = actual_sqlstate {
                writeln!(out, "actual_sqlstate: {sqlstate}")
                    .expect("writing to String should not fail");
            }
            writeln!(out, "actual_error: {err}").expect("writing to String should not fail");
        }
        TestErrorKind::StatementResultMismatch {
            expected, actual, ..
        } => {
            writeln!(out, "expected_rows: {expected}").expect("writing to String should not fail");
            writeln!(out, "actual_rows: {actual}").expect("writing to String should not fail");
        }
        TestErrorKind::Ok { .. }
        | TestErrorKind::Fail { .. }
        | TestErrorKind::SystemFail { .. }
        | TestErrorKind::SystemStdoutMismatch { .. }
        | TestErrorKind::ParseError(_)
        | _ => {
            writeln!(out, "details: {}", test_err.display(false))
                .expect("writing to String should not fail");
        }
    }

    out.trim_end_matches('\n').to_string()
}

#[cfg(test)]
mod tests {
    use super::{RecordMetadataCache, format_error, format_failed, format_ok};
    use sqllogictest::Record;

    #[test]
    fn status_formatters_plain_text() {
        assert_eq!(format_ok(false), "ok");
        assert_eq!(format_failed(false), "FAILED");
        assert_eq!(format_error(false), "ERROR");
    }

    #[test]
    fn status_formatters_colored_text() {
        assert!(format_ok(true).contains("ok"));
        assert!(format_failed(true).contains("FAILED"));
        assert!(format_error(true).contains("ERROR"));
    }

    #[test]
    fn metadata_cache_supports_repeated_lookups_for_same_file() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("duckdb-slt-reporting-{unique}.slt"));

        std::fs::write(
            &path,
            "query I first\nSELECT 1;\n----\n1\n\nquery I second\nSELECT 2;\n----\n2\n",
        )
        .expect("fixture should be written");

        let cache = RecordMetadataCache::build(&path).expect("metadata should be built");
        let records = sqllogictest::parse_file::<sqllogictest::DefaultColumnType>(&path)
            .expect("fixture should parse");
        let query_locs: Vec<_> = records
            .into_iter()
            .filter_map(|record| match record {
                Record::Query { loc, .. } => Some(loc),
                _ => None,
            })
            .collect();

        assert_eq!(query_locs.len(), 2);

        let first = cache
            .find(&query_locs[0])
            .expect("first record metadata should exist");
        assert_eq!(first.index_1_based, 1);
        assert_eq!(first.name.as_deref(), Some("first"));

        let second = cache
            .find(&query_locs[1])
            .expect("second record metadata should exist");
        assert_eq!(second.index_1_based, 2);
        assert_eq!(second.name.as_deref(), Some("second"));

        let repeated_first = cache
            .find(&query_locs[0])
            .expect("repeated lookup should return same metadata");
        assert_eq!(repeated_first, first);

        let _ = std::fs::remove_file(&path);
    }
}
