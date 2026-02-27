## 1. Preprocessor Pipeline Foundation

- [x] 1.1 Refactor `src/preprocessor.rs` to introduce a pipeline entrypoint that evaluates directive handlers in deterministic order.
- [x] 1.2 Define minimal internal handler interfaces/types for per-line directive matching, line transformation, and directive side effects.
- [x] 1.3 Preserve existing `PreprocessRun` and `PreprocessDirectives` external behavior through the new pipeline entrypoint.

## 2. Require Handler Extraction

- [x] 2.1 Implement a dedicated `require` directive handler that ports current token parsing behavior.
- [x] 2.2 Ensure the `require` handler rewrites matched lines as commented directives while preserving original line endings.
- [x] 2.3 Ensure required extensions collected by the handler map to the same runtime-loading behavior as before.

## 3. Line-Mapping and Transform Guarantees

- [x] 3.1 Enforce one-input-line to one-output-line transformation invariants in pipeline processing.
- [x] 3.2 Preserve pass-through behavior for non-directive/comment lines and preserve line ordering.
- [x] 3.3 Preserve newline style (`\n` and `\r\n`) for each corresponding output line.

## 4. Regression Coverage and Validation

- [x] 4.1 Add directive-specific unit tests for `require` matching/parsing/rewriting edge cases.
- [x] 4.2 Add preprocessor-level tests that assert line-count and line-content mapping invariants.
- [x] 4.3 Run `cargo test --locked` and fix regressions.
- [x] 4.4 Run `cargo clippy --locked -- -D warnings` and fix lint issues.
