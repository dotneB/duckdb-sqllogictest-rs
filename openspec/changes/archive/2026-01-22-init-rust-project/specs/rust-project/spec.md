## ADDED Requirements

### Requirement: Rust Edition and Minimum Supported Rust Version
The system SHALL define a Rust Cargo package named `duckdb-slt` with `edition = "2024"` and `package.rust-version = "1.85"`.

#### Scenario: Developer inspects the manifest for edition and MSRV
- **WHEN** a developer opens `Cargo.toml`
- **THEN** it declares `edition = "2024"` and `rust-version = "1.85"` for the `duckdb-slt` package

### Requirement: Toolchain Pinning
The system SHALL pin the Rust toolchain to version 1.85.0 using `rust-toolchain.toml`.

#### Scenario: Developer installs toolchain from repository configuration
- **WHEN** a developer runs `rustup show` (or Rust tooling reads `rust-toolchain.toml`)
- **THEN** the repository configuration selects Rust toolchain `1.85.0`

### Requirement: Cargo Resolver/MSRV Behavior (Edition 2024)
The system SHALL use Rust 2024 such that Cargo resolver v3 behavior is in effect, enabling Rust-version aware dependency resolution aligned with `package.rust-version`.

#### Scenario: Dependencies resolve compatibly with the declared Rust version
- **WHEN** a developer runs `cargo build` using Rust 1.85.0
- **THEN** Cargo resolves dependencies without selecting versions that require a newer Rust compiler than the declared `rust-version`

### Requirement: Cargo Project Skeleton
The system SHALL provide a Rust (edition 2024) Cargo project that builds a CLI binary named `duckdb-slt`.

#### Scenario: Developer builds the project
- **WHEN** a developer runs `cargo build`
- **THEN** the build completes successfully and produces the `duckdb-slt` binary artifact

### Requirement: Baseline Dependencies Declared
The system SHALL declare baseline Rust dependencies required for a DuckDB sqllogictest CLI runner: `clap`, `anyhow`, `duckdb`, and `sqllogictest`.

#### Scenario: Developer compiles with dependencies
- **WHEN** a developer runs `cargo build`
- **THEN** Cargo resolves and compiles the declared dependencies successfully

### Requirement: CI Enforcement via GitHub Actions
The system SHALL provide a GitHub Actions workflow at `.github/workflows/ci.yml` that enforces formatting, linting, and tests.

#### Scenario: CI runs required checks
- **WHEN** GitHub Actions runs the workflow on a push or pull request
- **THEN** it executes `cargo fmt --check`, `cargo clippy -- -D warnings`, and `cargo test` and fails the job if any command fails
