# Change: Implement Extension Spec Parsing and INSTALL/LOAD Execution

## Why
The CLI already accepts `--extensions <EXT>` but needs well-defined parsing and safe SQL generation so callers can reliably enable core, community, and local-file extensions without hand-writing SQL or risking malformed `INSTALL`/`LOAD` statements.

## What Changes
- Define and implement an extension spec grammar supporting `name`, `name@<repository>` (including DuckDB aliases and custom repository URLs), and `path/to/ext.duckdb_extension`.
- Generate `INSTALL` and `LOAD` SQL from parsed specs, including correct quoting/escaping for file paths.
- Execute extension `INSTALL`/`LOAD` actions (in CLI order) before running any sqllogictest files.
- Add unit tests for spec parsing and SQL generation.

## Impact
- Affected specs: `duckdb-slt-cli`.
- Affected code: new `src/extensions.rs` module; main runner wiring to apply extensions prior to executing tests.
- Security: reduces SQL-injection risk by ensuring extension specs are converted into quoted SQL literals (especially for file paths).
