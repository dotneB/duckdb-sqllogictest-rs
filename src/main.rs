mod duckdb_driver;
mod extensions;

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::{Context, Result};
use clap::{ArgAction, Parser, error::ErrorKind};
use duckdb::{Config, Connection};
use sqllogictest::runner::TestErrorKind;
use sqllogictest::{QueryExpect, Record, Runner};

use crate::duckdb_driver::DuckdbDriver;
use crate::extensions::ExtensionActions;

const EXIT_OK: u8 = 0;
const EXIT_RUNTIME_ERROR: u8 = 1;
const EXIT_TEST_FAIL: u8 = 2;

#[derive(Parser, Debug)]
#[command(name = "duckdb-slt", about = "DuckDB sqllogictest runner", version)]
struct Cli {
    /// Path to the DuckDB database file. Defaults to an in-memory database.
    #[arg(long, value_name = "PATH")]
    db: Option<PathBuf>,

    /// Allow loading unsigned DuckDB extensions (risky; opt-in).
    #[arg(long)]
    allow_unsigned_extensions: bool,

    /// DuckDB extensions to enable (repeatable). Each entry runs INSTALL then LOAD.
    #[arg(short = 'e', long, value_name = "EXT")]
    extensions: Vec<String>,

    /// Working directory to apply before resolving relative paths.
    #[arg(short = 'w', long, value_name = "DIR")]
    workdir: Option<PathBuf>,

    /// Stop after the first test mismatch.
    #[arg(long, default_value_t = true, action = ArgAction::SetTrue)]
    fail_fast: bool,

    /// Continue running remaining files after a mismatch.
    #[arg(long = "no-fail-fast", action = ArgAction::SetTrue)]
    no_fail_fast: bool,

    /// One or more sqllogictest input files.
    #[arg(value_name = "FILES", required = true)]
    files: Vec<PathBuf>,
}

fn main() -> ExitCode {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(err) => {
            return match err.kind() {
                ErrorKind::DisplayHelp | ErrorKind::DisplayVersion => {
                    print!("{err}");
                    ExitCode::SUCCESS
                }
                _ => {
                    eprintln!("{err}");
                    ExitCode::from(EXIT_RUNTIME_ERROR)
                }
            };
        }
    };

    match run(cli) {
        Ok(code) => ExitCode::from(code),
        Err(err) => {
            eprintln!("error: {err:?}");
            ExitCode::from(EXIT_RUNTIME_ERROR)
        }
    }
}

fn run(cli: Cli) -> Result<u8> {
    if let Some(workdir) = &cli.workdir {
        std::env::set_current_dir(workdir)
            .with_context(|| format!("failed to set workdir: {}", workdir.display()))?;
    }

    let fail_fast = if cli.no_fail_fast {
        false
    } else {
        cli.fail_fast
    };
    let files = expand_files(&cli.files)?
        .into_iter()
        .map(|p| normalize_path(&p))
        .collect::<Result<Vec<_>>>()?;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum FileStatus {
        Pass,
        Fail,
        Error,
    }

    let mut results: Vec<(FileStatus, String)> = Vec::new();
    let mut files_total = 0usize;
    let mut files_passed = 0usize;
    let mut files_failed = 0usize;
    let mut files_errored = 0usize;

    for path in files {
        files_total += 1;
        let res = run_one_file(&cli, &path);
        match res {
            Ok(()) => {
                files_passed += 1;
                results.push((FileStatus::Pass, path.display().to_string()));
            }
            Err(FileRunError::TestFailure(e)) => {
                eprintln!("{e}");
                files_failed += 1;
                results.push((FileStatus::Fail, path.display().to_string()));

                if fail_fast {
                    break;
                }
            }
            Err(FileRunError::Runtime(e)) => {
                eprintln!("{e:?}");
                files_errored += 1;
                results.push((FileStatus::Error, path.display().to_string()));
                // Runtime errors are not recoverable for a single run.
                break;
            }
        }
    }

    let exit_code = if files_errored > 0 {
        EXIT_RUNTIME_ERROR
    } else if files_failed > 0 {
        EXIT_TEST_FAIL
    } else {
        EXIT_OK
    };

    for (status, path) in &results {
        let label = match status {
            FileStatus::Pass => "PASS",
            FileStatus::Fail => "FAIL",
            FileStatus::Error => "ERROR",
        };
        println!("{label} {path}");
    }
    println!(
        "files: total={} passed={} failed={} errored={}",
        files_total, files_passed, files_failed, files_errored
    );

    Ok(exit_code)
}

fn expand_files(files: &[PathBuf]) -> Result<Vec<PathBuf>> {
    let mut out: Vec<PathBuf> = Vec::new();

    for p in files {
        if looks_like_glob_pattern(p) {
            let pattern = normalize_glob_pattern(p);
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
            out.push(p.clone());
        }
    }

    Ok(out)
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

fn normalize_path(path: &Path) -> Result<PathBuf> {
    let path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    };

    Ok(path)
}

enum FileRunError {
    TestFailure(String),
    Runtime(anyhow::Error),
}

#[derive(Debug, Clone)]
struct RecordId {
    index_1_based: usize,
    name: Option<String>,
}

fn find_record_id(main_file: &Path, loc: &sqllogictest::Location) -> Option<RecordId> {
    let file_hint = PathBuf::from(loc.file());
    let candidate = if file_hint.is_absolute() {
        file_hint
    } else {
        main_file
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(file_hint)
    };

    let records = sqllogictest::parse_file::<sqllogictest::DefaultColumnType>(&candidate).ok()?;
    let mut index = 0usize;

    for r in records {
        match r {
            Record::Statement { loc: rloc, .. } => {
                index += 1;
                if rloc.file() == loc.file() && rloc.line() == loc.line() {
                    return Some(RecordId {
                        index_1_based: index,
                        name: None,
                    });
                }
            }
            Record::System { loc: rloc, .. } => {
                index += 1;
                if rloc.file() == loc.file() && rloc.line() == loc.line() {
                    return Some(RecordId {
                        index_1_based: index,
                        name: None,
                    });
                }
            }
            Record::Query {
                loc: rloc,
                expected,
                ..
            } => {
                index += 1;
                if rloc.file() == loc.file() && rloc.line() == loc.line() {
                    let name = match expected {
                        QueryExpect::Results { label, .. } => label,
                        QueryExpect::Error(_) => None,
                    };
                    return Some(RecordId {
                        index_1_based: index,
                        name,
                    });
                }
            }
            _ => {}
        }
    }

    None
}

fn render_failure_report(main_file: &Path, test_err: &sqllogictest::TestError) -> String {
    use std::fmt::Write;

    let kind = test_err.kind();
    let loc = test_err.location();
    let record_id = find_record_id(main_file, &loc);

    let mut out = String::new();

    writeln!(out, "test mismatch").expect("writing to String should not fail");
    writeln!(out, "file: {}", loc.file()).expect("writing to String should not fail");
    writeln!(out, "at: {loc}").expect("writing to String should not fail");
    if let Some(id) = &record_id {
        writeln!(
            out,
            "record: {}{}",
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
            writeln!(out, "expected:\n{expected}").expect("writing to String should not fail");
            writeln!(out, "actual:\n{actual}").expect("writing to String should not fail");
        }
        TestErrorKind::QueryResultColumnsMismatch {
            expected, actual, ..
        } => {
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
            // Fallback: still include the underlying library error message.
            writeln!(out, "details: {}", test_err.display(false))
                .expect("writing to String should not fail");
        }
    }

    out.trim_end_matches('\n').to_string()
}

fn run_one_file(cli: &Cli, path: &Path) -> std::result::Result<(), FileRunError> {
    if !path.exists() {
        return Err(FileRunError::Runtime(anyhow::anyhow!(
            "file does not exist: {}",
            path.display()
        )));
    }

    let db = cli.db.clone();
    let allow_unsigned_extensions = cli.allow_unsigned_extensions;
    let extensions = cli
        .extensions
        .iter()
        .map(|raw| crate::extensions::compile_extension_actions(raw))
        .collect::<Result<Vec<ExtensionActions>>>()
        .map_err(FileRunError::Runtime)?;
    let mut runner = Runner::new(move || {
        let db = db.clone();
        let extensions = extensions.clone();

        async move {
            let conn = open_duckdb_connection(db.as_deref(), allow_unsigned_extensions)?;

            for ext in &extensions {
                eprintln!("INSTALL {}", ext.display);
                conn.execute_batch(&ext.install_sql)?;

                eprintln!("LOAD {}", ext.display);
                conn.execute_batch(&ext.load_sql)?;
            }

            Ok(DuckdbDriver::new(conn))
        }
    });

    match runner.run_file(path) {
        Ok(()) => Ok(()),
        Err(test_err) => match test_err.kind() {
            TestErrorKind::ParseError(_) => Err(FileRunError::Runtime(anyhow::anyhow!(
                "parse error in {}: {}",
                path.display(),
                test_err.display(false)
            ))),
            _ => Err(FileRunError::TestFailure(render_failure_report(
                path, &test_err,
            ))),
        },
    }
}

fn open_duckdb_connection(
    db: Option<&Path>,
    allow_unsigned_extensions: bool,
) -> duckdb::Result<Connection> {
    let mut config = Config::default();
    if allow_unsigned_extensions {
        config = config.allow_unsigned_extensions()?;
    }

    let conn = match db {
        Some(p) => Connection::open_with_flags(p, config)?,
        None => Connection::open_in_memory_with_flags(config)?,
    };

    Ok(conn)
}
