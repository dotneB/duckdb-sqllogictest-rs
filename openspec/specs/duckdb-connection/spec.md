# duckdb-connection Specification

## Purpose
Define the DuckDB connection initialization contract for `duckdb-slt`, including how database paths are handled and how security-sensitive settings (like allowing unsigned extensions) are applied.
## Requirements
### Requirement: DuckDB Connection Configuration
The system SHALL open DuckDB connections using `duckdb::Config`.

When `--db <PATH>` is not provided, the system SHALL open an in-memory DuckDB database.

#### Scenario: User does not provide --db
- **WHEN** a user runs the CLI without `--db`
- **THEN** the system opens an in-memory DuckDB database

#### Scenario: User provides --db
- **WHEN** a user runs the CLI with `--db path/to/db.duckdb`
- **THEN** the system opens (or creates) the database at `path/to/db.duckdb`

### Requirement: Unsigned Extension Opt-In
The system SHALL keep unsigned extensions disabled by default.

When `--allow-unsigned-extensions` is provided, the system SHALL enable DuckDB's unsigned extension support via `duckdb::Config`.

Enabling unsigned extensions SHALL NOT require emitting a warning banner.

#### Scenario: Unsigned extensions disabled by default
- **WHEN** a user runs the CLI without `--allow-unsigned-extensions`
- **THEN** the DuckDB connection is created without enabling unsigned extensions

#### Scenario: Unsigned extensions explicitly enabled
- **WHEN** a user runs the CLI with `--allow-unsigned-extensions`
- **THEN** the DuckDB connection is created with unsigned extensions enabled

#### Scenario: No banner is required when unsigned is enabled
- **WHEN** a user runs the CLI with `--allow-unsigned-extensions`
- **THEN** the system does not emit a warning banner solely due to enabling unsigned extensions
