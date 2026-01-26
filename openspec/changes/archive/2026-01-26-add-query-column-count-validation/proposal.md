# Change: Validate query column counts

## Why
`duckdb-slt` currently allows sqllogictest `query` directives to pass even when the declared column spec (e.g., `query III`) does not match the number of columns returned by the SQL query. This hides real test mistakes and differs from the expectations implied by the sqllogictest format.

## What Changes
- Enforce that the number of columns returned by a `query` matches the number of columns declared by the query type string (`I`, `T`, `R`, etc.).
- Improve mismatch diagnostics for column-count mismatches to clearly state expected vs actual column counts.

## Impact
- Affected specs: `openspec/specs/duckdb-slt-cli/spec.md`
- Affected code: `src/main.rs` (runner configuration + diagnostics)
- Affected tests: update fixtures that currently contain invalid column specs; add regression coverage for the mismatch case
