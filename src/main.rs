mod cli;
mod duckdb_driver;
mod extensions;
mod orchestrator;
mod pathing;
mod preprocessor;
mod reporting;
mod runtime;

use std::process::ExitCode;

use clap::error::ErrorKind;

use crate::orchestrator::EXIT_RUNTIME_ERROR;

fn main() -> ExitCode {
    let cli = match crate::cli::try_parse() {
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

    match crate::orchestrator::run(cli) {
        Ok(code) => ExitCode::from(code),
        Err(err) => {
            eprintln!("error: {err:?}");
            ExitCode::from(EXIT_RUNTIME_ERROR)
        }
    }
}
