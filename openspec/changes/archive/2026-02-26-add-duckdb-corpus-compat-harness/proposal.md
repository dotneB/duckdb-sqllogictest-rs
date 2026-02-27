## Why

Our current integration coverage relies mostly on hand-written fixtures, so regressions can still slip through when behavior diverges from DuckDB's real sqllogictest corpus. We need a reproducible compatibility harness and broader directive support now to validate real-world migration scenarios and keep CI confidence high.

## What Changes

- Add a compatibility harness that executes a curated, pinned subset of DuckDB's upstream sqllogictest corpus as part of local and CI validation.
- Add corpus-subset management (selection, pinning, and layout) so compatibility runs are deterministic and maintainable.
- Extend directive preprocessing/execution to support additional DuckDB compatibility directives required by the selected corpus subset (beyond existing `require` handling).
- Add regression tests that assert directive behavior and corpus-subset pass/fail expectations.

## Capabilities

### New Capabilities
- `duckdb-corpus-compat-harness`: Define how a real DuckDB sqllogictest corpus subset is curated, pinned, executed, and reported as a compatibility harness.

### Modified Capabilities
- `duckdb-slt-cli`: Expand DuckDB directive compatibility by extending the preprocessor/runner contract for additional directives used by the harness corpus subset.

## Impact

- Affected code: `src/preprocessor.rs`, runner/CLI orchestration modules, and integration/regression test fixtures.
- Affected tests: new corpus-subset harness tests and directive-focused regression tests.
- Affected tooling/ops: deterministic corpus-subset storage/update workflow and CI invocation for compatibility harness runs.
