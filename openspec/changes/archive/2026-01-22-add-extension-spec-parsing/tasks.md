## 1. Implementation
- [x] 1.1 Add `src/extensions.rs` with:
  - [x] 1.1.1 Extension spec parser (name, name@repository, .duckdb_extension path)
  - [x] 1.1.2 SQL generation for `INSTALL` + `LOAD` from parsed specs
  - [x] 1.1.3 Helpers for SQL string literal escaping and path detection
- [x] 1.2 Wire extension execution into the runner so all extension actions run before any sqllogictest file executes
- [x] 1.3 Ensure relative extension paths resolve against `--workdir` semantics

## 2. Testing
- [x] 2.1 Unit tests for spec parsing:
  - [x] 2.1.1 `json` (name)
  - [x] 2.1.2 `spatial@community`
  - [x] 2.1.3 `custom_extension@https://my-custom-extension-repository`
  - [x] 2.1.4 `path/to/ext.duckdb_extension`
  - [x] 2.1.5 Rejection cases (malformed spec)
- [x] 2.2 Unit tests for SQL generation:
  - [x] 2.2.1 Correct SQL for name/community/path specs
  - [x] 2.2.2 Path escaping for single quotes

## 3. Validation
- [x] 3.1 Run `cargo fmt`
- [x] 3.2 Run `cargo clippy -- -D warnings`
- [x] 3.3 Run `cargo test`
