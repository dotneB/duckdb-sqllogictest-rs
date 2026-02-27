## 1. Module scaffolding and boundaries

- [x] 1.1 Create `src/cli.rs`, `src/orchestrator.rs`, `src/reporting.rs`, `src/pathing.rs`, and `src/runtime.rs`, then wire module declarations from `src/main.rs`.
- [x] 1.2 Move the `Cli` argument model and clap-parse policy from `src/main.rs` into `src/cli.rs`.
- [x] 1.3 Move file input expansion, path normalization, and user-path display helpers into `src/pathing.rs`.
- [x] 1.4 Move DuckDB connection creation and extension bootstrap logic into `src/runtime.rs`.

## 2. Execution and reporting extraction

- [x] 2.1 Move the top-level run loop and file-level execution control into `src/orchestrator.rs`.
- [x] 2.2 Move mismatch rendering and status-formatting helpers into `src/reporting.rs`.
- [x] 2.3 Reduce `src/main.rs` to a thin entrypoint that delegates parse/run/report responsibilities to module APIs.
- [x] 2.4 Verify refactor parity for existing CLI behavior (flags, exit-code precedence, fail-fast/runtime stop policy, and output semantics).

## 3. Regression safety and validation

- [x] 3.1 Add or adjust targeted tests for extracted pure helpers where practical (especially pathing/reporting).
- [x] 3.2 Run `cargo test --locked` and resolve any regressions introduced by extraction.
- [x] 3.3 Run `cargo clippy --locked -- -D warnings` and resolve lint failures.
- [x] 3.4 Perform CLI smoke checks (`--help`, passing fixture, failing fixture) to confirm behavior parity after refactor.
