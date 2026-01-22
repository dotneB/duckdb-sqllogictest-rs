## ADDED Requirements

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
