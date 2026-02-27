## 1. Corpus Subset Foundation

- [x] 1.1 Add a pinned DuckDB sqllogictest corpus subset in-repo with metadata for upstream source revision and selected files.
- [x] 1.2 Define deterministic subset ordering/selection rules for harness execution.
- [x] 1.3 Document the subset refresh workflow so updates are auditable in version control.

## 2. Directive Pipeline Enhancements

- [x] 2.1 Refactor `src/preprocessor.rs` into an extensible directive handler pipeline while preserving current `require` behavior.
- [x] 2.2 Implement `skipif`/`onlyif` DuckDB-compatible conditional record handling required by the selected corpus subset.
- [x] 2.3 Preserve line-number stability across directive rewrites so downstream parse/mismatch diagnostics remain accurate.
- [x] 2.4 Add validation that fails compatibility runs when a required corpus directive is unsupported, including directive keyword and source location.

## 3. Runner Integration

- [x] 3.1 Update runner integration to consume expanded directive effects from preprocessing before test execution.
- [x] 3.2 Ensure preprocessed-path error reporting continues to map diagnostics back to original corpus files.

## 4. Compatibility Verification

- [x] 4.1 Add an integration harness target that runs the pinned DuckDB corpus subset through `duckdb-slt` and fails on mismatches.
- [x] 4.2 Add directive-focused regression fixtures/tests for `skipif`/`onlyif` behavior.
- [x] 4.3 Add regression coverage that asserts unsupported required directives fail with actionable errors.
- [x] 4.4 Wire the compatibility harness into CI/local validation commands and confirm runtime remains within agreed budget.
- [x] 4.5 `just dev`
- [x] 4.6 `just full`
