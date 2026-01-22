## 1. Implementation
- [x] 1.1 Centralize DuckDB connection creation behind a helper function (inputs: db path option, allow-unsigned flag)
- [x] 1.2 Wire `duckdb::Config` and open connection with flags (default to in-memory when `--db` is omitted)
- [x] 1.3 Implement `--allow-unsigned-extensions` by applying the DuckDB config option for unsigned extensions

## 2. Validation
- [x] 2.1 Add an integration test that opens DuckDB in-memory with unsigned extensions disabled
- [x] 2.2 Add an integration test that opens DuckDB in-memory with unsigned extensions enabled
- [x] 2.3 If feasible, assert the setting value via a DuckDB settings query (e.g., `duckdb_settings()`), otherwise limit the test to “opens successfully”
- [x] 2.4 Run `cargo fmt`, `cargo clippy -- -D warnings`, and `cargo test`
