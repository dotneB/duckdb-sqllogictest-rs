# AGENTS guide (`duckdb-sqllogictest-rs`)

Use this as the minimal operating guide for coding agents in this repo.

## Project
- Rust `edition = "2024"`
- Binary crate: `duckdb-slt`
- MSRV: `1.89.0`
- CI toolchain: `1.93.0`

## Cargo commands (when you need precision)
Build:
```bash
cargo build --locked --verbose
cargo build --release --locked --verbose
```

Lint/format:
```bash
cargo fmt -- --check
cargo fmt
cargo clippy -- -D warnings
```

Tests:
```bash
cargo test --locked --verbose
cargo test --test cli
cargo test --lib

# single integration test (exact)
cargo test --test cli pass_exits_0 -- --exact

# single unit test (exact path)
cargo test duckdb_driver::tests::map_duckdb_types -- --exact

# filtered subset
cargo test --test cli require_

# with output shown
cargo test --test cli mismatch_exits_2 -- --exact --nocapture
```

## Minimum validation before finishing

`just dev`

Which is equivalent of running:
1. `cargo fmt -- --check`
2. `cargo clippy -- -D warnings`
3. `cargo test --locked --verbose`

For narrow edits, run at least clippy + the most relevant focused test.

## Coding rules (keep these stable)
- Formatting: rustfmt defaults; keep functions focused; prefer early returns.
- Imports: group as `std`, external crates, `crate::...`; avoid wildcards.
- Visibility: private by default; `pub(crate)` for internal sharing; `pub` only when needed.
- Naming: `UpperCamelCase` types, `snake_case` funcs/modules/vars, `SCREAMING_SNAKE_CASE` consts.
- Types/ownership: prefer `&Path`/`&[T]` params unless ownership is required; clone intentionally.
- Errors: no `unwrap`/`expect` in production paths; use `anyhow::Result` + `Context`; include actionable context (paths/operations).
- Output contract: stdout/stderr is user interface; keep result text stable (`ok`, `FAILED`, `ERROR`).
- Exit codes are fixed: `0` success, `1` runtime/config/CLI error, `2` sqllogictest mismatch/failure.
- Testing: behavior-oriented names, deterministic cross-platform assertions, cover happy + failure paths.
- DuckDB/SQL: treat extension specs as untrusted, validate/normalize, escape SQL literals, keep formatting behavior stable.
- Cross-platform: support Linux/macOS/Windows; avoid hard-coded separators; use `std::path` helpers.

## High-value files
- `src/cli.rs` - CLI args/flags
- `src/main.rs` - entrypoint
- `src/orchestrator.rs` - run orchestration + exit paths
- `src/duckdb_driver.rs` - adapter + value formatting
- `src/extensions.rs` - extension parsing + SQL generation
- `src/preprocessor.rs` - `require` preprocessing
- `src/pathing.rs` - path expansion/display
- `src/reporting.rs` - failure rendering
- `tests/cli.rs` - integration test harness
- `.github/workflows/ci.yml` - CI truth source
