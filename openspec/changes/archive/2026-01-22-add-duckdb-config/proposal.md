# Change: Add DuckDB Connection Config and Unsigned Extension Support

## Why
The CLI runner needs deterministic and configurable DuckDB connection setup so that test runs are reproducible in CI and local development. Unsigned extensions are a security-sensitive opt-in and must be explicitly enabled to avoid accidental unsafe configurations.

## What Changes
- Open DuckDB connections using `duckdb::Config`, defaulting to an in-memory database when `--db` is not provided.
- Implement `--allow-unsigned-extensions` by enabling DuckDB's unsigned extension setting via `duckdb::Config`.
- Add a small integration test that opens DuckDB with and without unsigned extensions enabled.

## Impact
- Affected specs: new capability spec `duckdb-connection`.
- Affected code: DuckDB connection initialization (expected in `src/main.rs` or a small helper module).
- Dependencies: assumes CLI flags `--db` and `--allow-unsigned-extensions` exist (see change `add-cli-contract`).
