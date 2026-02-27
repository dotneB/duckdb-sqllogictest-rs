use std::io::{IsTerminal, Write};
use std::path::Path;

use anyhow::{Context, Result};
use sqllogictest::Runner;
use sqllogictest::runner::TestErrorKind;

use crate::cli::Cli;
use crate::pathing::{expand_files, format_user_path, normalize_path};
use crate::preprocessor::preprocess_file;
use crate::reporting::{
    RecordMetadataCache, format_error, format_failed, format_ok, render_failure_report,
};
use crate::runtime::{compile_extensions, open_driver};

pub(crate) const EXIT_OK: u8 = 0;
pub(crate) const EXIT_RUNTIME_ERROR: u8 = 1;
pub(crate) const EXIT_TEST_FAIL: u8 = 2;

#[allow(clippy::ptr_arg)]
fn validate_query_column_count(
    actual: &Vec<sqllogictest::DefaultColumnType>,
    expected: &Vec<sqllogictest::DefaultColumnType>,
) -> bool {
    actual.len() == expected.len()
}

enum FileRunError {
    TestFailure(String),
    Runtime(anyhow::Error),
}

pub(crate) fn run(cli: Cli) -> Result<u8> {
    if let Some(workdir) = &cli.workdir {
        std::env::set_current_dir(workdir)
            .with_context(|| format!("failed to set workdir: {}", workdir.display()))?;
    }

    let base_dir = std::env::current_dir()?;

    let files = expand_files(&cli.files)?
        .into_iter()
        .map(|path| normalize_path(&path))
        .collect::<Result<Vec<_>>>()?;

    println!("running {} tests", files.len());
    let started = std::time::Instant::now();
    let use_color = std::io::stdout().is_terminal();

    let mut files_passed = 0usize;
    let mut files_failed = 0usize;
    let mut files_errored = 0usize;

    let mut failed_files: Vec<String> = Vec::new();
    let mut errored_files: Vec<String> = Vec::new();

    for path in files {
        let display_path = format_user_path(&base_dir, &path);
        let res = run_one_file(&cli, &base_dir, &path);
        match res {
            Ok(()) => {
                files_passed += 1;
                println!("test {display_path} ... {}", format_ok(use_color));
            }
            Err(FileRunError::TestFailure(err)) => {
                files_failed += 1;
                failed_files.push(display_path.clone());
                println!("test {display_path} ... {}", format_failed(use_color));
                let _ = std::io::stdout().flush();
                eprintln!("{err}");

                if cli.fail_fast {
                    break;
                }
            }
            Err(FileRunError::Runtime(err)) => {
                files_errored += 1;
                errored_files.push(display_path.clone());
                println!("test {display_path} ... {}", format_error(use_color));
                let _ = std::io::stdout().flush();
                eprintln!("{err:?}");
                break;
            }
        }
    }

    if !failed_files.is_empty() || !errored_files.is_empty() {
        println!("\nfailures:\n");
        for file in &failed_files {
            println!("    {file}");
        }
        for file in &errored_files {
            println!("    {file}");
        }
        println!();
    }

    let exit_code = if files_errored > 0 {
        EXIT_RUNTIME_ERROR
    } else if files_failed > 0 {
        EXIT_TEST_FAIL
    } else {
        EXIT_OK
    };

    let status = if files_failed == 0 && files_errored == 0 {
        format_ok(use_color)
    } else {
        format_failed(use_color)
    };
    println!(
        "test result: {status}. {files_passed} passed; {files_failed} failed; {files_errored} errored; 0 ignored; 0 measured; 0 filtered out; finished in {:.2}s",
        started.elapsed().as_secs_f64()
    );

    Ok(exit_code)
}

fn run_one_file(cli: &Cli, base_dir: &Path, path: &Path) -> std::result::Result<(), FileRunError> {
    if !path.exists() {
        return Err(FileRunError::Runtime(anyhow::anyhow!(
            "file does not exist: {}",
            path.display()
        )));
    }

    let db = cli.db.clone();
    let allow_unsigned_extensions = cli.allow_unsigned_extensions;
    let extensions = compile_extensions(&cli.extensions).map_err(FileRunError::Runtime)?;

    let preprocessed = preprocess_file(path).map_err(FileRunError::Runtime)?;
    let run_path = preprocessed
        .as_ref()
        .map(|prep| prep.preprocessed_path().to_path_buf())
        .unwrap_or_else(|| path.to_path_buf());
    let record_metadata = RecordMetadataCache::build(&run_path);

    let required_extensions = preprocessed
        .as_ref()
        .map(|prep| prep.directives.required_extensions.clone())
        .unwrap_or_default();

    let mut runner = Runner::new(move || {
        let db = db.clone();
        let extensions = extensions.clone();
        let required_extensions = required_extensions.clone();

        async move {
            open_driver(
                db.as_deref(),
                allow_unsigned_extensions,
                &extensions,
                &required_extensions,
            )
        }
    });

    runner.with_column_validator(validate_query_column_count);

    match runner.run_file(&run_path) {
        Ok(()) => Ok(()),
        Err(test_err) => match test_err.kind() {
            TestErrorKind::ParseError(_) => {
                let mut details = test_err.display(false).to_string();
                if let Some(preprocessed) = preprocessed.as_ref() {
                    let from = preprocessed.preprocessed_path().to_string_lossy();
                    let to = path.to_string_lossy();
                    details = details.replace(from.as_ref(), to.as_ref());
                }

                Err(FileRunError::Runtime(anyhow::anyhow!(
                    "parse error in {}: {}",
                    path.display(),
                    details
                )))
            }
            _ => {
                let parse_main_file = preprocessed
                    .as_ref()
                    .map(|prep| prep.preprocessed_path())
                    .unwrap_or(path);

                Err(FileRunError::TestFailure(render_failure_report(
                    path,
                    parse_main_file,
                    base_dir,
                    &test_err,
                    record_metadata.as_ref(),
                )))
            }
        },
    }
}
