# duckdb-slt

DuckDB sqllogictest runner.

## Usage

Run one or more sqllogictest files:

```bash
duckdb-slt tests/fixtures/pass.slt
duckdb-slt suite/a.slt suite/b.slt
```

Enable extensions before running tests:

```bash
duckdb-slt --extensions json tests/fixtures/pass.slt
```

Short forms:

```bash
duckdb-slt -e json -w suite tests/fixtures/pass.slt
```

Show help and version:

```bash
duckdb-slt --help
duckdb-slt --version
```

## Exit Codes

- `0`: all tests passed
- `2`: at least one test failed due to an expectation mismatch
- `1`: runtime error (I/O error, DuckDB error, invalid configuration, invalid CLI usage)
