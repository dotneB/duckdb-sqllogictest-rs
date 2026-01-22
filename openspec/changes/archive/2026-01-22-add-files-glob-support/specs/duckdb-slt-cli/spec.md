## MODIFIED Requirements
### Requirement: CLI Surface (Flags and Positional Files)
The system SHALL provide a stable CLI for the `duckdb-slt` binary with the following arguments:

- `--db <PATH>`: path to the DuckDB database file (when omitted, the system uses an in-memory database).
- `--allow-unsigned-extensions`: opt-in flag to permit loading unsigned extensions.
- `--extensions <EXT>` / `-e <EXT>`: zero or more extension specs; may be repeated.
- `--workdir <DIR>` / `-w <DIR>`: base working directory for resolving relative paths.
- `--fail-fast` / `--no-fail-fast`: toggle whether execution stops after the first test mismatch.
- `--format <text|json>`: select output format.
- `<FILES...>`: one or more sqllogictest input files or glob patterns to execute.

The system SHALL default `--format` to `text` and SHALL default `fail-fast` behavior to enabled.

When `--workdir` is provided, the system SHALL resolve relative paths (including `<FILES...>` and `--db <PATH>`) using `--workdir` as the base.

The system SHALL expand glob patterns provided via `<FILES...>` into a list of matching files.

The system SHALL preserve file execution order by:
- expanding each `<FILES...>` argument from left-to-right, and
- sorting the matches of each glob pattern lexicographically before execution.

If a glob pattern matches zero files, the system SHALL treat it as a runtime error.

#### Scenario: User runs the CLI with required inputs
- **WHEN** a user runs `duckdb-slt path/to/test.slt`
- **THEN** the system executes the provided file using the default configuration

#### Scenario: User supplies multiple files
- **WHEN** a user runs `duckdb-slt a.slt b.slt`
- **THEN** the system executes the files in the given order

#### Scenario: User uses a glob pattern
- **WHEN** a user runs `duckdb-slt tests/fixtures/pass*.slt`
- **THEN** the system expands the glob and executes each matched file

#### Scenario: User uses multiple globs and paths
- **WHEN** a user runs `duckdb-slt a.slt tests/fixtures/pass*.slt b.slt`
- **THEN** the system executes `a.slt`, then the expanded glob matches in lexicographic order, then `b.slt`

#### Scenario: Glob matches zero files
- **WHEN** a user runs `duckdb-slt does-not-exist-*.slt`
- **THEN** the system exits with a runtime error

#### Scenario: User sets workdir for relative paths
- **WHEN** a user runs `duckdb-slt --workdir suite/ tests/a.slt`
- **THEN** the system resolves relative file paths using `suite/` as the base directory

#### Scenario: User disables fail-fast
- **WHEN** a user runs `duckdb-slt --no-fail-fast a.slt b.slt` and a mismatch occurs in `a.slt`
- **THEN** the system still attempts to execute `b.slt`
