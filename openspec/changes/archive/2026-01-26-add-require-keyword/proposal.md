# Change: Add support for `require` directives

## Why
DuckDB sqllogictest suites (and `duckdb-sqllogictest-python`) use the `require` keyword to ensure required extensions are available before running the rest of a test file. Today, `duckdb-slt` delegates parsing to the upstream `sqllogictest` crate, which rejects unknown keywords and therefore fails to run such suites.

## What Changes
- Add a compatibility layer that recognizes `require <EXT>` lines and attempts to load the referenced DuckDB extension before executing subsequent records.
- Make `require` lines parseable by `sqllogictest` without shifting line numbers (e.g., by preprocessing them into comment lines).
- The implementation does not attempt `INSTALL` for `require`; it only attempts `LOAD` to avoid guessing install locations.
- If a required extension cannot be loaded, treat `require` as a no-op (pass/ignored) and continue executing subsequent records (matching the observed behavior of DuckDB’s Python runner).

## Impact
- Affected specs: `openspec/specs/duckdb-slt-cli/spec.md`
- Affected code: `src/main.rs` (file execution orchestration), `src/preprocessor.rs` (directive preprocessing), `src/extensions.rs` (reuse existing extension spec parsing)
- Affected tests/docs: new/updated fixtures and CLI integration tests under `tests/`
