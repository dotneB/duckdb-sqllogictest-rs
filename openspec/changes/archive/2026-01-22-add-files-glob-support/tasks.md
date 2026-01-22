## 1. Implementation
- [x] 1.1 Implement glob expansion for `<FILES...>` arguments (support literal paths and glob patterns)
- [x] 1.2 Preserve file execution order: expand patterns left-to-right; sort each pattern's matches lexicographically for determinism
- [x] 1.3 Error when a glob pattern matches zero files (runtime error / exit code 1)
- [x] 1.4 Add an integration test that invokes the CLI with a glob pattern (e.g., `tests/fixtures/pass*.slt`) and asserts success

## 2. Validation
- [x] 2.1 Run `cargo test`
- [x] 2.2 Run `cargo fmt --check`
- [x] 2.3 Run `cargo clippy -- -D warnings`
