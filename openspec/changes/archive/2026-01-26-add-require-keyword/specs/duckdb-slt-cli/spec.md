## ADDED Requirements

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
