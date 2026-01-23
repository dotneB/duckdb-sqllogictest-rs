## 1. Implementation
- [x] 1.1 Remove the `advanced-type-formatting` cargo feature from `Cargo.toml` (including default feature wiring).
- [x] 1.2 Update `src/duckdb_driver.rs` to always compile and use canonical nested formatting; remove `#[cfg(feature = "advanced-type-formatting")]` branches and the debug-format fallback for nested types.
- [x] 1.3 Add/adjust tests so nested value rendering is covered without feature flags.
- [x] 1.4 Validate: `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`.

## 2. Spec Updates
- [x] 2.1 Update `openspec/specs/duckdb-slt-driver/spec.md` during archive to remove feature-gated formatting and make nested formatting unconditional.
- [x] 2.2 Run `openspec validate --strict --no-interactive` after archiving.
