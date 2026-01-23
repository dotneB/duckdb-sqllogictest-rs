# Change: Remove `advanced-type-formatting` feature gate

## Why
DuckDB suite compatibility requires canonical stringification for nested values (LIST/STRUCT/MAP). Keeping this behavior behind a cargo feature creates inconsistent output across builds and complicates usage.

## What Changes
- The driver will ALWAYS render nested values using DuckDB-compatible stringification.
- **BREAKING**: Remove the `advanced-type-formatting` cargo feature; callers can no longer enable/disable nested formatting via features.
- Remove OpenSpec requirements that describe feature-gated nested formatting.

## Impact
- Affected specs: `openspec/specs/duckdb-slt-driver/spec.md`
- Affected code: `Cargo.toml`, `src/duckdb_driver.rs`
