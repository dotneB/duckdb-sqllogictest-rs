use std::path::PathBuf;

use clap::{ArgAction, Parser};

#[derive(Parser, Debug)]
#[command(name = "duckdb-slt", about = "DuckDB sqllogictest runner", version)]
pub(crate) struct Cli {
    /// Path to the DuckDB database file. Defaults to an in-memory database.
    #[arg(long, value_name = "PATH")]
    pub(crate) db: Option<PathBuf>,

    /// Allow loading unsigned DuckDB extensions (risky; opt-in).
    #[arg(short = 'u', long)]
    pub(crate) allow_unsigned_extensions: bool,

    /// DuckDB extensions to enable (repeatable). Each entry runs INSTALL then LOAD.
    #[arg(short = 'e', long, value_name = "EXT")]
    pub(crate) extensions: Vec<String>,

    /// Working directory to apply before resolving relative paths.
    #[arg(short = 'w', long, value_name = "DIR")]
    pub(crate) workdir: Option<PathBuf>,

    /// Stop after the first test mismatch.
    #[arg(long, default_value_t = false, action = ArgAction::SetTrue)]
    pub(crate) fail_fast: bool,

    /// One or more sqllogictest input files.
    #[arg(value_name = "FILES", required = true)]
    pub(crate) files: Vec<PathBuf>,
}

pub(crate) fn try_parse() -> Result<Cli, clap::Error> {
    Cli::try_parse()
}
