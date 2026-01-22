## ADDED Requirements

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
