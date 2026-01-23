# Change: Update DuckDB Type Fidelity for Suite Compatibility

## Why
DuckDB sqllogictest suites commonly assert textual renderings of non-primitive DuckDB types (date/time/timestamps, intervals, decimals, and nested types). The current driver falls back to debug-style formatting for these values, which causes mismatches even when query semantics are correct.

## What Changes
- Expand driver value stringification to match DuckDB's canonical textual output for:
  - `DATE`, `TIME`, `TIMESTAMP`, `INTERVAL`, `DECIMAL`
  - Nested types: lists, structs, and maps
- Expand column type mapping so these types participate in sqllogictest comparisons as `Text` where appropriate.
- Add golden tests per type to lock in output compatibility against DuckDB.
- Add an opt-out feature flag for advanced (nested) type formatting if implementation complexity or dependency surface warrants it.

## Impact
- Affected specs: `openspec/specs/duckdb-slt-driver/spec.md`
- Affected code (expected): `src/duckdb_driver.rs`, `Cargo.toml`, `tests/`
- Risk: changing stringification can cause existing suites to start passing (desired) but could also change outputs for callers relying on debug formatting.
