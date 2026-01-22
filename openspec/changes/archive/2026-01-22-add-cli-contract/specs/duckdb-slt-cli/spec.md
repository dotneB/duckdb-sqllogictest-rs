## ADDED Requirements

### Requirement: CLI Surface (Flags and Positional Files)
The system SHALL provide a stable CLI for the `duckdb-slt` binary with the following arguments:

- `--db <PATH>`: path to the DuckDB database file (when omitted, the system uses an in-memory database).
- `--allow-unsigned-extensions`: opt-in flag to permit loading unsigned extensions.
- `--extensions <EXT>` / `-e <EXT>`: zero or more extension specs; may be repeated.
- `--workdir <DIR>` / `-w <DIR>`: base working directory for resolving relative paths.
- `--fail-fast` / `--no-fail-fast`: toggle whether execution stops after the first test mismatch.
- `--format <text|json>`: select output format.
- `<FILES...>`: one or more sqllogictest input files to execute.

The system SHALL default `--format` to `text` and SHALL default `fail-fast` behavior to enabled.

When `--workdir` is provided, the system SHALL resolve relative paths (including `<FILES...>` and `--db <PATH>`) using `--workdir` as the base.

#### Scenario: User runs the CLI with required inputs
- **WHEN** a user runs `duckdb-slt path/to/test.slt`
- **THEN** the system executes the provided file using the default configuration

#### Scenario: User supplies multiple files
- **WHEN** a user runs `duckdb-slt a.slt b.slt`
- **THEN** the system executes the files in the given order

#### Scenario: User sets workdir for relative paths
- **WHEN** a user runs `duckdb-slt --workdir suite/ tests/a.slt`
- **THEN** the system resolves relative file paths using `suite/` as the base directory

#### Scenario: User disables fail-fast
- **WHEN** a user runs `duckdb-slt --no-fail-fast a.slt b.slt` and a mismatch occurs in `a.slt`
- **THEN** the system still attempts to execute `b.slt`

### Requirement: Extension Actions (Install and Load)
The system SHALL support extension actions prior to running tests.

#### Scenario: User enables an extension
- **WHEN** a user passes one or more `--extensions <EXT>` flags
- **THEN** the system executes `INSTALL` then `LOAD` for each extension in command-line order

#### Scenario: User enables multiple extensions
- **WHEN** a user runs `duckdb-slt --extensions a --extensions b test.slt`
- **THEN** the system executes `INSTALL a; LOAD a; INSTALL b; LOAD b` prior to running tests

#### Scenario: User enables extensions with short flag
- **WHEN** a user runs `duckdb-slt -e json test.slt`
- **THEN** the system executes `INSTALL json; LOAD json` prior to running tests

### Requirement: Output Formats
The system SHALL support `--format text` and `--format json`.

When `--format json` is selected, the system SHALL write a single JSON document to stdout that includes:
- `status`: `"pass" | "fail" | "error"`
- `exit_code`: `0 | 1 | 2`
- `counts`: `{ files_total, files_passed, files_failed, files_errored }`

In JSON mode, any human-readable diagnostics SHALL be written to stderr.

#### Scenario: User uses text output
- **WHEN** a user runs the CLI without specifying `--format`
- **THEN** the system prints human-readable progress and failures

#### Scenario: User uses JSON output
- **WHEN** a user runs `duckdb-slt --format json ...`
- **THEN** the system emits a machine-readable JSON document to stdout summarizing the run

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

### Requirement: CLI Help and Version Documentation
The system SHALL document `--help` and `--version` usage examples in `README.md`.

#### Scenario: Developer looks up CLI help
- **WHEN** a developer reads `README.md`
- **THEN** it includes an example invocation of `duckdb-slt --help`

#### Scenario: Developer looks up CLI version
- **WHEN** a developer reads `README.md`
- **THEN** it includes an example invocation of `duckdb-slt --version`
