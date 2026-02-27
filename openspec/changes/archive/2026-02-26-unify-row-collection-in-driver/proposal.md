## Why

The DuckDB sqllogictest driver currently collects rows through duplicated code paths. That duplication makes type refinement and value formatting easier to drift over time, which increases regression risk when one path is updated without the other.

## What Changes

- Consolidate row collection into one shared path used by all result-producing driver flows.
- Keep type refinement and canonical value formatting in a single source of truth.
- Add coverage to verify equivalent output for query-style execution and execute-with-results fallback.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- duckdb-slt-driver: require all result-producing execution paths to use the same row collection, type mapping, and canonical formatting pipeline.

## Impact

- Affected spec: `duckdb-slt-driver`
- Affected code: DuckDB sqllogictest driver row-collection and result-conversion logic
- No intended CLI surface changes; externally visible behavior remains compatible but more consistent
- Reduces maintenance overhead and lowers risk of formatting/type-regression bugs
