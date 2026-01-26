## 1. Implementation
- [x] 1.1 Add regression coverage for `TIMETZ` canonical output (expected includes offset).
- [x] 1.2 Update `src/duckdb_driver.rs` so time zone aware values stringify like `CAST(value AS VARCHAR)`.
- [x] 1.3 Run the full fixture suite and update any affected expected outputs.

## 2. Validation
- [x] 2.1 `cargo fmt --check`
- [x] 2.2 `cargo clippy -- -D warnings`
- [x] 2.3 `cargo test`
