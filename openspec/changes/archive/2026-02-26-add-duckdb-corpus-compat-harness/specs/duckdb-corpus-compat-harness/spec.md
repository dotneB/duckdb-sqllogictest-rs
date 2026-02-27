## ADDED Requirements

### Requirement: Pinned DuckDB Corpus Subset
The project SHALL include a checked-in, curated subset of DuckDB's sqllogictest corpus with metadata that pins the upstream source revision used to derive the subset.

#### Scenario: Subset metadata is reproducible
- **WHEN** a maintainer inspects the subset metadata
- **THEN** it includes the upstream source identifier (repository and commit/tag) and the selected test-file set used by the harness

#### Scenario: Harness execution is offline-deterministic
- **WHEN** the compatibility harness runs in CI or local development
- **THEN** it executes only the pinned local subset and does not require runtime network access

### Requirement: Corpus Compatibility Harness
The test suite SHALL provide a compatibility harness entry point that runs the pinned DuckDB corpus subset through `duckdb-slt` and fails on mismatches.

#### Scenario: Harness reports compatibility regressions
- **WHEN** a corpus subset file produces an expectation mismatch
- **THEN** the harness fails and surfaces file-scoped mismatch diagnostics

#### Scenario: Harness success is CI-gating
- **WHEN** all selected corpus subset files pass
- **THEN** the harness exits successfully and is eligible for CI gating

### Requirement: Corpus Subset Refresh Workflow
The repository SHALL define a documented refresh workflow for updating the pinned corpus subset while preserving deterministic reviewability.

#### Scenario: Subset refresh produces auditable changes
- **WHEN** the subset is updated to a newer upstream revision
- **THEN** the update includes metadata/version changes and file diffs that can be reviewed in version control
