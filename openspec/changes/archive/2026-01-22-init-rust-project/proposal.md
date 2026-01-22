# Change: Initialize Rust project skeleton

## Why
The repository currently contains no Rust crate, so there is nothing to build, test, or evolve toward the project goal (a DuckDB sqllogictest runner). Establishing a minimal Cargo project unblocks incremental implementation and CI integration.

## What Changes
- Add a minimal Cargo project skeleton with `Cargo.toml` and `src/main.rs`.
- Set crate/CLI name to `duckdb-slt`.
- Use Rust `edition = "2024"`.
- Enforce Rust >= 1.85 for the project by:
  - Pinning toolchain to Rust 1.85.0 via `rust-toolchain.toml`
  - Declaring `package.rust-version = "1.85"` in `Cargo.toml`
- Add baseline dependencies needed for the planned CLI runner (no functional behavior required in this change): `clap`, `anyhow`, `duckdb`, `sqllogictest`.
- Add GitHub Actions CI workflow that runs formatting, linting, and tests.

## Notes (Edition 2024 + Resolver/MSRV)
- Rust 2024 implies Cargo resolver v3, which is Rust-version aware. With `package.rust-version = "1.85"`, dependency resolution is expected to prefer dependency versions compatible with the declared Rust version (instead of selecting versions that require a newer compiler and failing later).

## Acceptance Criteria
- `Cargo.toml` declares `edition = "2024"` and `rust-version = "1.85"` for the `duckdb-slt` package.
- `rust-toolchain.toml` pins Rust toolchain `1.85.0`.
- `.github/workflows/ci.yml` exists and runs:
  - `cargo fmt --check`
  - `cargo clippy -- -D warnings`
  - `cargo test`
- On a clean checkout with Rust 1.85.0 installed, `cargo build` and `cargo test` succeed.

## Impact
- Affected specs: `rust-project` (new)
- Affected code: `Cargo.toml`, `rust-toolchain.toml`, `src/main.rs`, `.github/workflows/ci.yml`
- Risks: Introduces native dependency (`duckdb`) which may require system toolchain support; mitigate via early build validation in CI.
