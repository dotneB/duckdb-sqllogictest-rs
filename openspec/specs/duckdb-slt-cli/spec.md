# duckdb-slt-cli Specification

## Purpose
Define the stable, CI-friendly command-line interface contract for the `duckdb-slt` binary, including supported flags, extension pre-run behavior, output formats, and exit codes.
## Requirements
### Requirement: CLI Surface (Flags and Positional Files)
The system SHALL provide a stable CLI for the `duckdb-slt` binary with the following arguments:

- `--db <PATH>`: path to the DuckDB database file (when omitted, the system uses an in-memory database).
- `--allow-unsigned-extensions`: opt-in flag to permit loading unsigned extensions.
- `--extensions <EXT>` / `-e <EXT>`: zero or more extension specs; may be repeated.
- `--workdir <DIR>` / `-w <DIR>`: base working directory for resolving relative paths.
- `--fail-fast`: toggle whether execution stops after the first test mismatch.
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

### Requirement: CLI Help and Version Documentation
The system SHALL document `--help` and `--version` usage examples in `README.md`.

#### Scenario: Developer looks up CLI help
- **WHEN** a developer reads `README.md`
- **THEN** it includes an example invocation of `duckdb-slt --help`

#### Scenario: Developer looks up CLI version
- **WHEN** a developer reads `README.md`
- **THEN** it includes an example invocation of `duckdb-slt --version`

### Requirement: Extension Spec Parsing
When `--extensions <EXT>` is provided, the system SHALL parse each `<EXT>` entry using the following supported forms:

- `name` (e.g., `json`)
- `name@repository` (e.g., `spatial@community`, `httpfs@core_nightly`, `custom_extension@https://my-custom-extension-repository`)
- `path/to/ext.duckdb_extension` (a local extension file)

#### Scenario: Parse extension name
- **WHEN** a user passes `--extensions json`
- **THEN** the system treats the spec as an extension name

#### Scenario: Parse repository extension name
- **WHEN** a user passes `--extensions spatial@community`
- **THEN** the system treats the spec as an extension name installed from the specified repository

#### Scenario: Parse custom repository URL
- **WHEN** a user passes `--extensions custom_extension@https://my-custom-extension-repository`
- **THEN** the system treats the spec as an extension name installed from the specified repository

#### Scenario: Parse local extension path
- **WHEN** a user passes `--extensions path/to/ext.duckdb_extension`
- **THEN** the system treats the spec as a local extension file path

#### Scenario: Reject malformed extension spec
- **WHEN** a user passes an empty extension spec
- **THEN** the system exits with a runtime error

### Requirement: Extension SQL Generation
The system SHALL generate and execute DuckDB `INSTALL` and `LOAD` statements for each parsed extension spec.

Generated SQL that embeds a file path SHALL quote the path as a SQL string literal.

The system SHALL escape single quotes inside generated SQL string literals.

#### Scenario: Generate SQL for name extension
- **WHEN** a user passes `--extensions json`
- **THEN** the system executes `INSTALL 'json';` then `LOAD 'json';`

#### Scenario: Generate SQL for community extension
- **WHEN** a user passes `--extensions spatial@community`
- **THEN** the system executes `INSTALL 'spatial' FROM community;` then `LOAD 'spatial';`

#### Scenario: Generate SQL for custom repository URL
- **WHEN** a user passes `--extensions custom_extension@https://my-custom-extension-repository`
- **THEN** the system executes `INSTALL 'custom_extension' FROM 'https://my-custom-extension-repository';` then `LOAD 'custom_extension';`

#### Scenario: Generate SQL for local extension path
- **WHEN** a user passes `--extensions path/to/ext.duckdb_extension`
- **THEN** the system executes `INSTALL 'path/to/ext.duckdb_extension';` then `LOAD 'path/to/ext.duckdb_extension';`

#### Scenario: Escape single quotes in local extension path
- **WHEN** a user passes `--extensions path/with'quote/ext.duckdb_extension`
- **THEN** the generated SQL escapes the single quote within the string literal

### Requirement: Mismatch Failure Diagnostics
When an expectation mismatch occurs while running `duckdb-slt`, the system SHALL write a human-readable failure report to stderr that includes:
- the input file path
- a record identifier (record index; and record name when available)
- the SQL statement (or a recognizable snippet of it)
- the expected output
- the actual output

For each input file execution, the system SHALL parse record metadata for mismatch diagnostics at most once and SHALL reuse that metadata for every mismatch reported for the same file.

Record metadata reuse SHALL preserve the same record identifier and SQL context that would be produced by location-based lookup.

#### Scenario: Query mismatch prints file, record identifier, and SQL
- **WHEN** a query record mismatches between expected and actual output
- **THEN** stderr includes the file path, the record identifier, and the SQL statement for the failing record

#### Scenario: Statement mismatch prints expected vs actual
- **WHEN** a statement record mismatches due to an unexpected error or unexpected success
- **THEN** stderr includes both the expected outcome and the actual outcome

#### Scenario: Multiple mismatches in one file reuse metadata cache
- **WHEN** two or more mismatches are reported while executing the same sqllogictest file
- **THEN** record metadata is parsed once for that file and reused for each mismatch report

### Requirement: User-Facing File Paths
When `duckdb-slt` prints a user-facing file path (e.g., PASS/FAIL lines and mismatch reports), it SHALL prefer paths relative to the process working directory (after applying `--workdir`).

If a relative path cannot be computed reliably, the system SHALL fall back to printing the absolute path.

#### Scenario: PASS output uses a relative path
- **WHEN** a user runs `duckdb-slt --workdir suite/ sql/test.slt`
- **THEN** the tool prints `PASS sql\\test.slt` (or equivalent platform separator) instead of an absolute path

#### Scenario: FAIL output uses a relative path
- **WHEN** a mismatch occurs while running `sql/fail.slt` under `--workdir suite/`
- **THEN** the tool prints `FAIL sql\\fail.slt` using a relative path

### Requirement: Failure Report Placement
When a mismatch occurs in a file, the system SHALL print the `FAIL <file>` status line before printing the corresponding mismatch failure report.

#### Scenario: Failure details appear under FAIL
- **WHEN** a mismatch occurs while running `sql/fail.slt`
- **THEN** the output order is `FAIL sql\\fail.slt` followed by lines such as `at:` / `record:` / `sql:`

### Requirement: DuckDB `require` Directives
The system SHALL support DuckDB-style `require` directives in sqllogictest input files.

A `require` directive SHALL be a non-empty line whose first non-whitespace token is `require`.

When a `require` directive is present, the system SHALL treat the remainder of the line as an extension name and SHALL attempt to execute `LOAD` for that extension prior to executing any subsequent records in the file.

The system SHOULD accept both `require <name>` and `require '<name>'` forms.

If the system cannot satisfy a `require` directive (e.g., the extension cannot be installed or loaded), the system SHALL treat the directive as a no-op and continue executing subsequent records in the file.

#### Scenario: Required extension loads and test runs
- **WHEN** a sqllogictest file contains `require parquet` followed by statements that use the extension
- **THEN** the system executes `LOAD 'parquet'` before running subsequent records

#### Scenario: Multiple require directives load in order
- **WHEN** a file contains multiple `require` directives
- **THEN** the system attempts to load required extensions in the order they appear

#### Scenario: Required extension cannot be loaded
- **WHEN** a file contains `require some_extension` and the extension cannot be installed or loaded
- **THEN** the system continues executing records after the `require` directive and any extension-dependent statements fail normally

### Requirement: `require` Compatibility Preprocessing
Because the upstream `sqllogictest` parser rejects unknown keywords, the system SHALL preprocess `require` directive lines into sqllogictest comment lines before invoking the upstream parser.

Preprocessing SHALL preserve the original line numbering of the remaining records.

#### Scenario: `require` lines do not cause parse errors
- **WHEN** a file contains one or more `require` lines
- **THEN** the file is accepted for parsing by the upstream `sqllogictest` crate

#### Scenario: Failure locations keep original line numbers
- **WHEN** a mismatch occurs after a `require` line
- **THEN** the reported `at: <file>:<line>` line refers to the original file line numbering

### Requirement: Extensible Directive Preprocessor
The system SHALL implement test-file directive preprocessing in a dedicated, extensible preprocessor module.

The system SHALL implement the `require` directive using this preprocessor.

When new directive keywords are introduced in the future, the system SHALL add them by extending the preprocessor module.

#### Scenario: Add a new directive keyword
- **WHEN** a future change adds support for an additional directive keyword
- **THEN** that directive is implemented by extending `src/preprocessor.rs`

### Requirement: Additional DuckDB Compatibility Directives
The CLI preprocessing pipeline SHALL support the additional DuckDB compatibility directives required by the pinned corpus subset, including `skipif` and `onlyif` conditions.

#### Scenario: `skipif duckdb` skips the following record
- **WHEN** preprocessing encounters `skipif duckdb` before a record
- **THEN** the directive and its associated record are excluded from execution for the DuckDB runner

#### Scenario: `onlyif` non-DuckDB target skips the following record
- **WHEN** preprocessing encounters `onlyif <target>` where `<target>` is not `duckdb`
- **THEN** the directive and its associated record are excluded from execution for the DuckDB runner

#### Scenario: `onlyif duckdb` preserves execution
- **WHEN** preprocessing encounters `onlyif duckdb` before a record
- **THEN** the associated record remains executable by `sqllogictest`

### Requirement: Directive Rewrite Line Stability
Directive preprocessing SHALL preserve source line numbering so mismatch and parse diagnostics continue to report stable file and line locations.

#### Scenario: Rewritten directives keep line count stable
- **WHEN** preprocessing rewrites compatibility directives into parser-safe lines
- **THEN** the generated script preserves original line count and relative line mapping for downstream error reporting

### Requirement: Required Directive Coverage Validation
Compatibility runs SHALL fail with an actionable runtime error if a corpus-selected file contains a directive required by the harness but not yet implemented in the preprocessor pipeline.

#### Scenario: Unsupported required directive is reported
- **WHEN** a selected corpus file contains an unsupported required directive
- **THEN** the run fails with the directive keyword and source location in the error output

### Requirement: Query Column Count Validation
For each `query` record, the system SHALL validate that the number of columns returned by DuckDB matches the number of columns declared by the query type string (e.g., `query III` declares 3 columns).

When the returned column count does not match the declared column count, the system SHALL fail the test.

#### Scenario: Query declares more columns than returned
- **WHEN** a test contains `query III` and the SQL returns 1 column
- **THEN** the test fails with a column-count mismatch

#### Scenario: Query declares fewer columns than returned
- **WHEN** a test contains `query I` and the SQL returns 2 columns
- **THEN** the test fails with a column-count mismatch

### Requirement: Column Count Mismatch Diagnostics
When a test fails due to a query column-count mismatch, the system SHALL include in its failure report:
- the expected column count
- the actual column count

#### Scenario: Failure report includes expected and actual counts
- **WHEN** a query fails due to a column-count mismatch
- **THEN** stderr includes text equivalent to `Expected X columns, but got Y columns`

### Requirement: Focused CLI Implementation Modules
The system SHALL organize CLI execution logic into focused internal modules with the following responsibilities:

- `src/cli.rs`: argument model and parse policy.
- `src/orchestrator.rs`: test-run orchestration, fail-fast/runtime policy, and summary accounting.
- `src/reporting.rs`: mismatch rendering and human-readable report formatting.
- `src/pathing.rs`: input file expansion, path normalization, and user-facing path display logic.
- `src/runtime.rs`: DuckDB connection creation and extension bootstrap steps.

#### Scenario: Developer inspects module boundaries
- **WHEN** a developer inspects the CLI runtime implementation in `src/`
- **THEN** each responsibility area is implemented in its corresponding focused module

### Requirement: Thin Binary Entrypoint Delegation
The system SHALL keep `src/main.rs` as a thin entrypoint that delegates parsing, orchestration, path handling, runtime setup, and reporting to focused modules while preserving existing user-visible CLI behavior.

#### Scenario: Existing CLI behavior remains unchanged after refactor
- **WHEN** the existing CLI integration test suite is executed after module extraction
- **THEN** command-line behavior, output semantics, and exit-code behavior remain consistent with pre-refactor expectations
