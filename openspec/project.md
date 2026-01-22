# Project Context

## Purpose

Build a small, CI-friendly Rust CLI that runs DuckDB SQL logic tests using **the same expectations model as DuckDB’s Python `sqltestrunner`** to minimize migration friction. The tool must be able to:

* Execute one or more sqllogictest files (`.test`, `.slt`, etc.) in-order against DuckDB
* Validate query results and statement outcomes using sqllogictest semantics (pass/fail)
* Optionally install/load DuckDB extensions prior to running tests
* Optionally allow **unsigned extensions** via an explicit, opt-in configuration

Primary use cases:

* **CI:** run test suites, fail non-zero on mismatch
* **Local dev:** run a single file with helpful failure output and reproducible behavior

Non-goals (v1):

* Re-implement sqllogictest parsing/comparison logic (use existing crate)
* Full parity with every DuckDB internal harness feature (parallelism, full environment matrices, etc.)

---

## Tech Stack

* Rust (edition 2021)
* DuckDB embedded database via `duckdb` / `duckdb-rs`
* sqllogictest runner via `sqllogictest` crate
* CLI parsing via `clap`
* Error handling via `anyhow`
* Optional: `glob`/`wax` for file globs, `tempfile` for test dir support, `regex` for any custom matching

---

## Project Conventions

### Code Style

* Format with `rustfmt` (default settings). Enforce with CI.
* Lint with `clippy` (deny warnings in CI where reasonable).
* Prefer explicit, readable code over cleverness.
* Naming:

  * Modules: `snake_case`
  * Types: `PascalCase`
  * Functions/vars: `snake_case`
* Error handling:

  * Use `anyhow::Result` at boundaries (main/CLI), and typed errors only if it materially improves UX.
  * Include context on errors (`.with_context(...)`) especially for file paths, SQL snippets, and extension actions.
* Logging/output:

  * Default to clean, human-readable output.
  * Print failures with enough context to debug: file, record index/name, SQL, expected vs actual.

### Architecture Patterns

Keep it simple and layered:

* `src/main.rs`

  * CLI parsing
  * DuckDB configuration (DB path, allow unsigned extensions)
  * Optional extension install/load
  * File discovery (including globs if supported)
  * Runner orchestration and exit codes
* `src/duckdb_driver.rs`

  * Implements `sqllogictest::DB` for DuckDB
  * Responsible for:

    * Executing SQL
    * Returning `DBOutput::StatementComplete` for statements
    * Returning `DBOutput::Rows` for queries (including type mapping and stringification)
* `src/extensions.rs` (optional)

  * Extension spec parsing (e.g., `name`, `name@community`, `/path/to/ext.duckdb_extension`)
  * SQL generation for `INSTALL` and `LOAD`
* `src/pathing.rs` (optional)

  * Resolving `<FILE>:` includes relative to suite root/workdir
  * Handling `__TEST_DIR__` substitution if implemented

Design principles:

* Prefer using `sqllogictest` crate functionality over custom behavior.
* Any DuckDB-specific compatibility tweaks should be isolated (ideally in driver/stringification layer).

### Testing Strategy

* Unit tests:

  * Extension spec parsing → correct generated SQL
  * Value formatting/stringification for key DuckDB types (NULL, empty string, ints, floats, text)
  * Type mapping from DuckDB/Arrow to sqllogictest column types (`I`, `R`, `T`, `Any`)
* Integration tests:

  * Run a small `.slt` fixture from `tests/fixtures/` and assert pass
  * Include at least one failure fixture and assert the CLI exits with the failure code and prints expected diagnostics
* Regression tests:

  * Keep a small set of representative DuckDB-style tests that cover:

    * `statement ok`
    * `statement error` (+ optional message matching)
    * `query` with `nosort/rowsort/valuesort`
    * `NULL` and `(empty)`
    * `<REGEX>:` / `<!REGEX>:`
    * `<FILE>:` include (if supported)
    * `__TEST_DIR__` substitution (if supported)

CI expectations:

* `cargo fmt --check`
* `cargo clippy -- -D warnings` (or a curated lint set)
* `cargo test`

### Git Workflow

* Trunk-based development recommended:

  * `main` is always green
  * Feature branches merged via PR
* Commit conventions:

  * Small commits with clear intent
  * Prefer conventional-ish prefixes when useful: `feat:`, `fix:`, `refactor:`, `test:`, `docs:`, `chore:`
* PR expectations:

  * Add/adjust tests for behavioral changes
  * Update `README.md` when CLI flags/behavior changes

---

## Domain Context

This project targets the **sqllogictest** model used widely in database testing, and specifically the variant used by DuckDB’s test suite (via Python `sqltestrunner`). Tests are typically written as records like:

* `statement ok` / `statement error`
* `query <colspec> [sortmode]` with expected rows after `----`

Key compatibility concerns:

* String-based comparison semantics: many assertions compare textual renderings of values.
* Special expected tokens such as `NULL` and `(empty)` are common.
* Tests may use regex expectations for plan text or non-deterministic strings.
* Sort modes (`nosort`, `rowsort`, `valuesort`) impact comparison behavior.
* Some suites include expected output via `<FILE>:` includes.
* DuckDB extension behavior may require explicit `INSTALL`/`LOAD`, and loading unsigned extensions is gated behind an opt-in setting.

---

## Important Constraints

* **Minimal migration friction:** prioritize behavior matching DuckDB’s existing test expectations over new formats.
* **Determinism:** output and comparisons should be stable in CI.
* **Security:** unsigned extensions must be **explicitly opt-in** and clearly marked as risky.
* **No custom sqllogictest parser in v1:** rely on the `sqllogictest` crate for parsing and result comparison where possible.
* **Exit codes are contract:** ensure stable codes for pass/fail/error so CI can rely on them.

Recommended exit code policy:

* `0` all tests passed
* `2` at least one test failed (mismatch)
* `1` runtime error (I/O, DuckDB error, invalid input, etc.)

---

## External Dependencies

* DuckDB embedded engine via `duckdb-rs` (links against DuckDB)
* `sqllogictest` crate (parsing + runner + comparison semantics)
* Optional dependencies:

  * `glob`/`wax` for file patterns
  * `tempfile` for generating/owning a per-run `__TEST_DIR__`
  * `serde_json` if adding structured output (`--format json`) for CI tooling

Optional integrations (future):

* JUnit output for CI test reporting
* GitHub Actions workflow examples for running suites in PRs
