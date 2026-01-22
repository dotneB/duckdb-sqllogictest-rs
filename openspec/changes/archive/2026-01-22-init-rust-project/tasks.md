## 1. Project Initialization
- [x] 1.1 Create `Cargo.toml` for package `duckdb-slt` using `edition = "2024"` and `package.rust-version = "1.85"`.
- [x] 1.2 Create `rust-toolchain.toml` pinning Rust toolchain to `1.85.0` (include `rustfmt` and `clippy` components as needed for CI).
- [x] 1.3 Create `src/main.rs` with a minimal `fn main()` and placeholder CLI parsing entrypoint (no runner behavior).
- [x] 1.4 Add baseline dependencies (`clap`, `anyhow`, `duckdb`, `sqllogictest`) with conservative version constraints.

## 2. CI (GitHub Actions)
- [x] 2.1 Add `.github/workflows/ci.yml` that checks out the repo, installs Rust 1.85.0, and runs:
  - [x] `cargo fmt --check`
  - [x] `cargo clippy -- -D warnings`
  - [x] `cargo test`

## 3. Validation (Local)
- [x] 3.1 `cargo fmt --check`
- [x] 3.2 `cargo clippy -- -D warnings`
- [x] 3.3 `cargo test`
