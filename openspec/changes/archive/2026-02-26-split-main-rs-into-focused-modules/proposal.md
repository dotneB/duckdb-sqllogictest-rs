## Why

`src/main.rs` currently carries CLI parsing, path expansion, runtime setup, test orchestration, and failure reporting in one large file. This makes behavior-preserving changes harder to reason about and raises regression risk as new features are added.

## What Changes

- Refactor the binary internals by splitting `src/main.rs` into focused modules: `cli.rs`, `orchestrator.rs`, `reporting.rs`, `pathing.rs`, and `runtime.rs`.
- Move existing logic into those modules without changing the user-facing CLI contract, exit-code semantics, or failure-report content.
- Keep `main.rs` as a thin entrypoint that parses arguments, delegates execution, and sets process exit behavior.
- Add/adjust tests around extracted module boundaries to lock in behavior during and after refactor.

## Capabilities

### New Capabilities
- *(none)*

### Modified Capabilities
- `duckdb-slt-cli`: require the CLI runner implementation to use focused internal modules with clear ownership boundaries while preserving existing observable behavior.

## Impact

- Affected code: `src/main.rs` (reduced), new `src/cli.rs`, `src/orchestrator.rs`, `src/reporting.rs`, `src/pathing.rs`, `src/runtime.rs`.
- Affected tests: integration tests in `tests/cli.rs` remain the primary behavior guard; additional targeted module tests are expected.
- APIs/dependencies: no new external dependencies; no intentional CLI breaking changes.
