## ADDED Requirements

### Requirement: Mismatch Failure Diagnostics
When an expectation mismatch occurs while running `duckdb-slt`, the system SHALL write a human-readable failure report to stderr that includes:
- the input file path
- a record identifier (record index; and record name when available)
- the SQL statement (or a recognizable snippet of it)
- the expected output
- the actual output

#### Scenario: Query mismatch prints file, record identifier, and SQL
- **WHEN** a query record mismatches between expected and actual output
- **THEN** stderr includes the file path, the record identifier, and the SQL statement for the failing record

#### Scenario: Statement mismatch prints expected vs actual
- **WHEN** a statement record mismatches due to an unexpected error or unexpected success
- **THEN** stderr includes both the expected outcome and the actual outcome

## MODIFIED Requirements

### Requirement: CLI Surface (Flags and Positional Files)
The system SHALL provide a stable CLI for the `duckdb-slt` binary with the following arguments:

- `--db <PATH>`: path to the DuckDB database file (when omitted, the system uses an in-memory database).
- `--allow-unsigned-extensions`: opt-in flag to permit loading unsigned extensions.
- `--extensions <EXT>` / `-e <EXT>`: zero or more extension specs; may be repeated.
- `--workdir <DIR>` / `-w <DIR>`: base working directory for resolving relative paths.
- `--fail-fast` / `--no-fail-fast`: toggle whether execution stops after the first test mismatch.
- `<FILES...>`: one or more sqllogictest input files or glob patterns to execute.

The system SHALL default `fail-fast` behavior to enabled.

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

### Requirement: Stable Exit Codes
The system SHALL use the following exit codes:

- `0` when all provided tests pass
- `2` when at least one test fails due to an expectation mismatch
- `1` when execution fails due to a runtime error (including I/O errors, DuckDB errors, invalid configuration, or invalid CLI usage)

#### Scenario: All tests pass
- **WHEN** all executed files complete with no mismatches
- **THEN** the process exits with code `0`

#### Scenario: A test mismatch occurs
- **WHEN** a mismatch occurs between expected and actual results
- **THEN** the process exits with code `2`

#### Scenario: Mismatch with no-fail-fast still exits 2
- **WHEN** a mismatch occurs and `--no-fail-fast` is enabled
- **THEN** the process exits with code `2` after attempting remaining files

#### Scenario: A runtime error occurs
- **WHEN** the system encounters an error that prevents correct execution (such as an unreadable input file)
- **THEN** the process exits with code `1`

#### Scenario: User requests help
- **WHEN** a user runs `duckdb-slt --help`
- **THEN** the process exits with code `0`

#### Scenario: User requests version
- **WHEN** a user runs `duckdb-slt --version`
- **THEN** the process exits with code `0`

#### Scenario: Invalid CLI usage
- **WHEN** a user runs `duckdb-slt --unknown-flag`
- **THEN** the process exits with code `1`

#### Scenario: No files provided
- **WHEN** a user runs `duckdb-slt` with no `<FILES...>` arguments
- **THEN** the process exits with code `1`

## REMOVED Requirements

### Requirement: Output Formats
**Reason**: Remove the `--format` flag and JSON summary output to simplify the CLI contract and rely on the runner's text output and optional external reporting.
**Migration**: Remove usage of `--format` in scripts; parse human-readable output or adopt runner-compatible reporting outside of `duckdb-slt`.
