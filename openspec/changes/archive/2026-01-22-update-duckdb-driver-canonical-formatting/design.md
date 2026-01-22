## Context
`duckdb-slt` integrates DuckDB with the `sqllogictest` crate by providing an implementation of `sqllogictest::DB`.

sqllogictest comparisons are based on:
- per-column declared type (`I`, `R`, `T`, or `Any`), and
- per-cell canonical string representations.

This means the driver layer must provide deterministic, CI-stable output.

## Goals / Non-Goals
- Goals:
  - Provide a clear execution contract for `DuckdbDriver::run` that distinguishes statements from queries.
  - Provide canonical stringification for the core DuckDB value set used by DuckDB sqllogictest suites.
  - Provide a stable type mapping contract for query result columns.
- Non-Goals:
  - Full rendering parity for every DuckDB type (e.g., complex/nested/struct/list/map) in this change.
  - Introducing new output modes or changing sqllogictest comparison semantics.

## Decisions
- Execution routing:
  - Use `Connection::prepare(sql)`.
  - Call `Statement::execute([])` first.
  - If execution indicates results are returned (e.g., an `ExecuteReturnedResults` error/variant), fall back to `Statement::query([])` and collect rows.
  - Otherwise treat the call as a statement and return `DBOutput::StatementComplete(rows_changed)`.

- Column type inference:
  - Prefer statement/metadata-provided column types when available.
  - If column types cannot be obtained without reading a row, infer from the first row's `ValueRef::data_type()` (current behavior), and fall back to `DefaultColumnType::Any` when metadata is not available.
  - Provide a mapping function that can accept both DuckDB type enums (e.g., `duckdb::types::Type`) and Arrow schema types (e.g., `arrow_schema::DataType`) so tests can cover both.

- Canonical value formatting:
  - `NULL` renders as the literal `NULL`.
  - Text renders as the exact UTF-8 string returned by DuckDB; the empty string renders as `(empty)`.
  - Integer/unsigned integer types render in base-10 with Rust `to_string()`.
  - Floating point renders using Rust `Display` (locale-independent) as a minimal, deterministic baseline.
  - Blob renders as lower-case hex with a `0x` prefix.

## Risks / Trade-offs
- DuckDB Rust API surface can differ across versions (e.g., how "execute returned results" is signaled). This change constrains itself to a minimal, version-friendly flow and keeps behavior isolated to `src/duckdb_driver.rs`.
- Float formatting can be a source of expectation mismatches for edge values; this proposal scopes to Rust `Display` initially, with follow-up changes possible if specific DuckDB suites require normalization.

## Migration Plan
- Update driver behavior and add unit tests.
- Run `cargo test` and ensure existing CLI integration tests (if any) continue to pass.

## Open Questions
- If DuckDB returns `Decimal` as a dedicated value type, should it be treated as `FloatingPoint` or `Text` for sqllogictest comparisons? This proposal treats it as `FloatingPoint` unless a DuckDB suite requires otherwise.
