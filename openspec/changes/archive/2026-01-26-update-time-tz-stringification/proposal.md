# Change: Fix TIME WITH TIME ZONE stringification

## Why
`duckdb-slt` currently formats DuckDB `TIME WITH TIME ZONE` (e.g., `TIMETZ '12:00:00+00:00'`) as `12:00:00`, dropping the offset. This allows tests to pass even when expected values include the time zone suffix (e.g., `12:00:00+00`), and violates the driver contract that date/time-like values match DuckDB stringification equivalent to `CAST(value AS VARCHAR)`.

## What Changes
- Update the DuckDB driver value formatting so `TIME WITH TIME ZONE` (and related time zone aware time/timestamp types) stringify identically to DuckDB’s `CAST(... AS VARCHAR)` output.
- Add regression coverage in fixtures to ensure time zone offsets are preserved.

## Impact
- Affected specs: `openspec/specs/duckdb-slt-driver/spec.md`
- Affected code: `src/duckdb_driver.rs`
- Affected tests: `tests/fixtures/canonical_values.slt` (and any new fixture coverage)
