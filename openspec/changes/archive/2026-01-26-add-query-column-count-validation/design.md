## Context
The upstream `sqllogictest` crate supports column validation via a configurable `ColumnTypeValidator`, but its default validator intentionally does not validate columns at all. `duckdb-slt` currently uses this default, so mismatched column counts can incorrectly pass.

## Goals / Non-Goals
- Goals:
  - Fail a `query` when the number of returned columns does not match the query type string length.
  - Produce a clear, user-facing error that reports expected vs actual column counts.
- Non-Goals:
  - Enforce column *types* in the initial change (DuckDB’s suite typically uses types as a column-count spec; type checking can be added later if needed).

## Decisions
- Use a custom column validator that checks only `actual.len() == expected.len()`.
  - This enables column-count validation without making type checks stricter than current DuckDB practice.
- Continue using the upstream `sqllogictest` runner and error types; map the column-mismatch error into a clearer `duckdb-slt` failure report.

## Risks / Trade-offs
- Existing fixtures/tests that relied on the lax behavior will now fail and must be corrected.
