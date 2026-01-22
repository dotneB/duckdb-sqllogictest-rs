mod duckdb_driver;
mod extensions;

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::{Context, Result};
use clap::{ArgAction, Parser, ValueEnum, error::ErrorKind};
use duckdb::{Config, Connection};
use serde::Serialize;
use sqllogictest::Runner;
use sqllogictest::runner::TestErrorKind;

use crate::duckdb_driver::DuckdbDriver;
use crate::extensions::ExtensionActions;

const EXIT_OK: u8 = 0;
const EXIT_RUNTIME_ERROR: u8 = 1;
const EXIT_TEST_FAIL: u8 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum OutputFormat {
    Text,
    Json,
}

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

    /// Output format.
    #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
    format: OutputFormat,

    /// One or more sqllogictest input files.
    #[arg(value_name = "FILES", required = true)]
    files: Vec<PathBuf>,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum RunStatus {
    Pass,
    Fail,
    Error,
}

#[derive(Debug, Serialize)]
struct JsonCounts {
    files_total: usize,
    files_passed: usize,
    files_failed: usize,
    files_errored: usize,
}

#[derive(Debug, Serialize)]
struct JsonFileResult {
    path: String,
    status: RunStatus,
}

#[derive(Debug, Serialize)]
struct JsonSummary {
    status: RunStatus,
    exit_code: u8,
    files: Vec<JsonFileResult>,
    counts: JsonCounts,
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

    let mut results: Vec<JsonFileResult> = Vec::with_capacity(files.len());

    for path in files {
        let res = run_one_file(&cli, &path);
        match res {
            Ok(()) => {
                results.push(JsonFileResult {
                    path: path.display().to_string(),
                    status: RunStatus::Pass,
                });
            }
            Err(FileRunError::TestFailure(e)) => {
                if cli.format == OutputFormat::Text {
                    eprintln!("{e}");
                }
                results.push(JsonFileResult {
                    path: path.display().to_string(),
                    status: RunStatus::Fail,
                });

                if fail_fast {
                    break;
                }
            }
            Err(FileRunError::Runtime(e)) => {
                if cli.format == OutputFormat::Text {
                    eprintln!("{e:?}");
                }
                results.push(JsonFileResult {
                    path: path.display().to_string(),
                    status: RunStatus::Error,
                });
                // Runtime errors are not recoverable for a single run.
                break;
            }
        }
    }

    let counts = JsonCounts {
        files_total: results.len(),
        files_passed: results
            .iter()
            .filter(|f| f.status == RunStatus::Pass)
            .count(),
        files_failed: results
            .iter()
            .filter(|f| f.status == RunStatus::Fail)
            .count(),
        files_errored: results
            .iter()
            .filter(|f| f.status == RunStatus::Error)
            .count(),
    };

    let (status, exit_code) = if counts.files_errored > 0 {
        (RunStatus::Error, EXIT_RUNTIME_ERROR)
    } else if counts.files_failed > 0 {
        (RunStatus::Fail, EXIT_TEST_FAIL)
    } else {
        (RunStatus::Pass, EXIT_OK)
    };

    match cli.format {
        OutputFormat::Text => {
            for f in &results {
                let label = match f.status {
                    RunStatus::Pass => "PASS",
                    RunStatus::Fail => "FAIL",
                    RunStatus::Error => "ERROR",
                };
                println!("{label} {}", f.path);
            }
            println!(
                "files: total={} passed={} failed={} errored={}",
                counts.files_total, counts.files_passed, counts.files_failed, counts.files_errored
            );
        }
        OutputFormat::Json => {
            let summary = JsonSummary {
                status,
                exit_code,
                files: results,
                counts,
            };
            println!("{}", serde_json::to_string(&summary)?);
        }
    }

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
    let format = cli.format;

    let mut runner = Runner::new(move || {
        let db = db.clone();
        let extensions = extensions.clone();

        async move {
            let conn = open_duckdb_connection(db.as_deref(), allow_unsigned_extensions)?;

            for ext in &extensions {
                if format == OutputFormat::Text {
                    eprintln!("INSTALL {}", ext.display);
                }
                conn.execute_batch(&ext.install_sql)?;

                if format == OutputFormat::Text {
                    eprintln!("LOAD {}", ext.display);
                }
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
            _ => Err(FileRunError::TestFailure(
                test_err.display(false).to_string(),
            )),
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
