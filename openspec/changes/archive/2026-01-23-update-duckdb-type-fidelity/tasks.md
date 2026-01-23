## 1. Implementation
- [x] 1.1 Identify DuckDB types and `duckdb::types::ValueRef` variants used for DATE/TIME/TIMESTAMP/INTERVAL/DECIMAL and nested types in the current `duckdb` crate version.
- [x] 1.2 Update driver column type mapping so DATE/TIME/TIMESTAMP/INTERVAL and nested types map to `DefaultColumnType::Text` (leaving unknown/unsupported types as `Any`; DECIMAL remains `DefaultColumnType::FloatingPoint`).
- [x] 1.3 Update driver value stringification to match DuckDB textual output for DATE/TIME/TIMESTAMP/INTERVAL/DECIMAL.
- [x] 1.4 Implement nested type (list/struct/map) formatting that matches DuckDB output.
- [x] 1.5 Add a cargo feature flag (default-enabled) that can disable nested type formatting and fall back to the prior behavior.

## 2. Tests
- [x] 2.1 Add golden tests for DATE/TIME/TIMESTAMP/INTERVAL/DECIMAL formatting using values sourced from DuckDB queries.
- [x] 2.2 Add golden tests for list/struct/map formatting (feature-enabled) and fallback behavior (feature-disabled).
- [x] 2.3 Ensure existing unit tests remain valid; adjust only where behavior is intentionally updated.

## 3. Validation
- [x] 3.1 Run `cargo test`.
- [x] 3.2 Run `cargo fmt --check`.
- [x] 3.3 Run `cargo clippy -- -D warnings`.
