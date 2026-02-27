## MODIFIED Requirements

### Requirement: DuckDB sqllogictest Driver
The system SHALL provide a DuckDB-backed implementation of `sqllogictest::DB` used by `duckdb-slt`.

The driver SHALL execute input SQL and return either statement completion or query results using sqllogictest runner types.

The driver SHALL collect query results through a single shared row-collection pipeline for all result-producing execution paths.

The shared row-collection pipeline SHALL be the single source of truth for per-column type refinement and canonical value formatting.

#### Scenario: Driver executes a statement
- **WHEN** the runner calls `DB::run` with a SQL statement that does not return rows
- **THEN** the driver returns `DBOutput::StatementComplete(rows_changed)`

#### Scenario: Driver executes a query
- **WHEN** the runner calls `DB::run` with a SQL query that returns rows
- **THEN** the driver returns `DBOutput::Rows { types, rows }` containing all returned rows and per-column types

#### Scenario: Driver distinguishes query from statement
- **WHEN** DuckDB indicates results are available during execution (e.g., "execute returned results")
- **THEN** the driver falls back to querying and collecting rows through the shared row-collection pipeline

#### Scenario: Type refinement and formatting are path-independent
- **WHEN** equivalent result sets are collected through different result-producing execution paths
- **THEN** both paths produce identical inferred column types and identical canonical string values
