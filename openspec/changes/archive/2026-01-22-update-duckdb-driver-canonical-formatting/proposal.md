# Change: Update DuckDB sqllogictest driver for canonical results

## Why
The sqllogictest harness is string-comparison driven, so DuckDB value rendering and column type inference must be deterministic and aligned with DuckDB's expectations model to avoid flaky CI failures.

## What Changes
- Update the DuckDB `sqllogictest::DB` implementation to follow a prepare/execute/query flow that correctly distinguishes statements from queries.
- Add a canonical `ValueRef -> String` rendering contract for core DuckDB types (NULL, text, numbers, bool, blob).
- Define and test a stable column type mapping contract from DuckDB (and Arrow schema types where available) to `sqllogictest::DefaultColumnType`.

## Impact
- Affected specs:
  - `duckdb-slt-driver` (new capability; driver execution, type mapping, value formatting)
- Affected code:
  - `src/duckdb_driver.rs`
  - Test modules under `src/duckdb_driver.rs` and/or `tests/`
- Behavior changes:
  - BLOB values will render as hex (`0x...`) instead of Rust debug bytes.
  - Some previously `Any`-typed columns may become `Integer`, `FloatingPoint`, or `Text` depending on available type metadata.
