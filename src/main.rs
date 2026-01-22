use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "duckdb-slt",
    about = "DuckDB sqllogictest runner (WIP)",
    version
)]
struct Cli {
    /// Optional path to a sqllogictest script (WIP)
    file: Option<PathBuf>,
}

fn main() -> Result<()> {
    let _cli = Cli::parse();

    // Touch key dependencies so `cargo build` in CI compiles them.
    let _ = std::mem::size_of::<duckdb::Connection>();
    let _ = sqllogictest::parser::parse_with_name::<sqllogictest::DefaultColumnType>("", "stdin");

    Ok(())
}
