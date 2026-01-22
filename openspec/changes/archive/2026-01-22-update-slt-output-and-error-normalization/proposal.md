# Change: Update duckdb-slt output and error normalization

## Why
Running third-party or extension-backed DuckDB sqllogictest suites (e.g., duckdb-chess) exposes environment-specific output that makes failures noisy and brittle in CI: absolute paths leak machine directories, failure reports print before the corresponding FAIL line, and DuckDB error strings differ across OS/runtime.

## What Changes
- Normalize select DuckDB error messages into stable, portable strings suitable for sqllogictest `expected_error` matching.
- Print per-file PASS/FAIL/ERROR lines using paths relative to the working directory (after applying `--workdir`).
- Print mismatch failure details directly under the corresponding `FAIL <file>` line (instead of appearing earlier in the output stream).

## Impact
- Affected specs: `openspec/specs/duckdb-slt-cli/spec.md`, `openspec/specs/duckdb-slt-driver/spec.md`
- Affected code (expected): `src/main.rs`, `src/duckdb_driver.rs`, `tests/cli.rs`
- Compatibility: changes output formatting and error-string matching behavior to improve cross-platform stability.
