use std::path::Path;

use anyhow::Result;
use duckdb::{Config, Connection};

use crate::duckdb_driver::{DuckdbDriver, DuckdbDriverError};
use crate::extensions::{ExtensionActions, compile_extension_actions, escape_sql_string_literal};

pub(crate) fn compile_extensions(raw_specs: &[String]) -> Result<Vec<ExtensionActions>> {
    raw_specs
        .iter()
        .map(|raw| compile_extension_actions(raw))
        .collect()
}

pub(crate) fn open_driver(
    db: Option<&Path>,
    allow_unsigned_extensions: bool,
    extensions: &[ExtensionActions],
    required_extensions: &[String],
) -> Result<DuckdbDriver, DuckdbDriverError> {
    let conn = open_duckdb_connection(db, allow_unsigned_extensions)?;

    for ext in extensions {
        conn.execute_batch(&ext.install_sql)?;
        conn.execute_batch(&ext.load_sql)?;
    }

    for name in required_extensions {
        let sql = format!("LOAD '{}';", escape_sql_string_literal(name));
        let _ = conn.execute_batch(&sql);
    }

    Ok(DuckdbDriver::new(conn))
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
        Some(path) => Connection::open_with_flags(path, config)?,
        None => Connection::open_in_memory_with_flags(config)?,
    };

    Ok(conn)
}
