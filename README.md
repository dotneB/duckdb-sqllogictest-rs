# duckdb-slt

A Rust-based sqllogictest runner for DuckDB.

[![Latest Version](https://img.shields.io/crates/v/duckdb-slt)](https://crates.io/crates/duckdb-slt)
[![MIT License](https://img.shields.io/crates/l/duckdb-slt)](LICENSE)
[![Downloads](https://img.shields.io/crates/d/duckdb-slt)](https://crates.io/crates/duckdb-slt)
[![CI](https://github.com/dotneB/duckdb-sqllogictest-rs/workflows/CI/badge.svg)](https://github.com/dotneB/duckdb-sqllogictest-rs/actions/workflows/ci.yml)

## Install

```bash
cargo install duckdb-slt
```

Or using binary install:

```bash
cargo binstall duckdb-slt
```

## Usage

Run one or more sqllogictest files:

```bash
duckdb-slt tests/fixtures/pass.slt
duckdb-slt suite/a.slt suite/b.slt
```

Run using a glob pattern

```bash
duckdb-slt "tests/fixtures/pass*.slt"
```

Set a working directory for resolving relative paths:

```bash
duckdb-slt --workdir suite tests/a.slt
```

Enable extensions before running tests:

```bash
duckdb-slt --extensions json tests/fixtures/pass.slt
```

Short forms:

```bash
duckdb-slt -e json -w suite tests/fixtures/pass.slt
```

Run an extension's integration tests (example extension: `quack`):

```bash
duckdb-slt.exe -e ./target/release/quack.duckdb_extension -u -w "$(CURDIR)" "$(CURDIR)/test/sql/*.test"
```

Show help and version:

```bash
duckdb-slt --help
duckdb-slt --version
```

## CLI

Options:

- `--db <PATH>`: DuckDB database file path (defaults to in-memory)
- `-u, --allow-unsigned-extensions`: allow loading unsigned DuckDB extensions (risky; opt-in)
- `-e, --extensions <EXT>`: enable extensions before running tests (repeatable)
- `-w, --workdir <DIR>`: set working directory before resolving relative paths
- `--fail-fast`: stop after the first test mismatch

## Extensions

Each `--extensions <EXT>` entry runs `INSTALL` then `LOAD`, in command-line order.

Supported forms:

- `json` (name)
- `spatial@community` (named repository)
- `custom_extension@https://my-extension-repo.example` (custom repository URL)
- `path/to/ext.duckdb_extension` (local extension file)

## Compatibility Notes

Not all keywords/directives added in [duckdb-sqllogictest-python](https://github.com/duckdb/duckdb-sqllogictest-python) are supported.
`duckdb-slt` at the moment only supports:
- `require` with the notable distinction that it only attempts to LOAD the extension, the installation of it needs to be done by using the [extensions option](#extensions).

## Exit Codes

- `0`: all tests passed
- `2`: at least one test failed due to an expectation mismatch
- `1`: runtime error (I/O error, DuckDB error, invalid configuration, invalid CLI usage)

## License

MIT. See `LICENSE`.
