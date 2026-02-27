## Context

`duckdb-slt` currently centralizes CLI parsing, file/path handling, runner orchestration, runtime setup, and failure rendering inside `src/main.rs`. The implementation works, but the concentration of concerns increases maintenance risk and makes behavior-preserving changes harder to review.

The change must preserve existing user-facing behavior (flags, output semantics, and exit codes), avoid new external dependencies, and keep compatibility with the existing integration test suite.

## Goals / Non-Goals

**Goals:**
- Split `src/main.rs` into focused modules with explicit ownership boundaries.
- Preserve observable CLI behavior and error/reporting semantics.
- Improve maintainability and testability by separating orchestration, pathing, runtime setup, and reporting.

**Non-Goals:**
- Introduce new CLI flags or change existing flag semantics.
- Change DuckDB driver value formatting behavior or test expectation rules.
- Redesign extension policy or `require` directive semantics.

## Decisions

1. **Adopt a five-module decomposition plus thin entrypoint**
   - `cli.rs`: argument model and parse policy.
   - `orchestrator.rs`: run loop, fail-fast/runtime handling, and final summary.
   - `reporting.rs`: mismatch rendering and human-readable output helpers.
   - `pathing.rs`: glob expansion, path normalization, user-facing display path logic.
   - `runtime.rs`: DuckDB connection setup and extension bootstrap.
   - `main.rs`: minimal glue for argument parsing, top-level error handling, and process exit.
   - **Alternative considered:** keep `main.rs` largely intact and extract only utility functions; rejected because it does not materially reduce coupling.

2. **Keep behavior-compatible API boundaries between modules**
   - Extract current logic into module-local functions with signatures that preserve current flow and error mapping.
   - Maintain the same fail-fast/runtime termination policy and exit-code precedence.
   - **Alternative considered:** redesign the execution flow while refactoring; rejected to minimize regression risk.

3. **Preserve reporting semantics exactly while relocating code**
   - Move mismatch formatting code into `reporting.rs` without changing output fields/order.
   - Keep path rendering behavior consistent with current relative-path preference.
   - **Alternative considered:** rewrite reporting with a new schema; rejected because it would bundle product changes into a structural refactor.

4. **Use existing integration tests as behavior lock and add focused module tests where high-value**
   - Keep `tests/cli.rs` as the primary guardrail for external behavior.
   - Add targeted tests for extracted pure helpers where practical.
   - **Alternative considered:** rely only on compile success and manual verification; rejected because it is insufficient for a sensitive refactor.

## Risks / Trade-offs

- **[Risk] Behavior drift during extraction** -> **Mitigation:** preserve function logic first, then improve structure incrementally; run full test suite after each extraction stage.
- **[Risk] Module dependency tangles** -> **Mitigation:** enforce one-way dependency direction (`main` -> `orchestrator` -> supporting modules).
- **[Risk] Larger short-term diff size** -> **Mitigation:** stage moves in small, reviewable commits with tests green at each step.

## Migration Plan

- No end-user migration is required; this is an internal refactor.
- Implement extraction in phases (pathing/runtime/reporting/orchestration) and keep tests passing throughout.
- Rollback strategy: revert the refactor commit(s) if behavior regressions are detected.

## Open Questions

- Should a follow-up change introduce `src/lib.rs` for reusable orchestration APIs, or keep all APIs binary-local for now?
  - Binary only
