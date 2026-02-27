# preprocessor-pipeline Specification

## Purpose
Define the stable contract for directive preprocessing so handler composition, `require` behavior, source mapping, and regression coverage remain consistent as new directives are added.

## Requirements
### Requirement: Composable Directive Pipeline
The preprocessor SHALL evaluate directive handlers through a deterministic, composable pipeline so directive logic can be extended without rewriting the preprocessing entrypoint.

#### Scenario: Ordered handler evaluation
- **WHEN** a non-comment input line is preprocessed
- **THEN** handlers are evaluated in configured order and at most one handler owns and transforms that line

### Requirement: Require Directive Handler Parity
The preprocessor SHALL implement `require` through a dedicated directive handler that preserves current behavior for extension extraction and line rewriting.

#### Scenario: Require directive is rewritten and captured
- **WHEN** preprocessing encounters a valid `require` directive line
- **THEN** the output line is rewritten as a commented form of the original directive and the required extension is recorded in preprocessing directives

### Requirement: Stable Source Line Mapping
The preprocessor SHALL preserve source-line mapping invariants used by downstream parser and failure reporting.

#### Scenario: One input line maps to one output line
- **WHEN** a script is preprocessed
- **THEN** the transformed script preserves the original line count and line ordering

#### Scenario: Original newline style is preserved
- **WHEN** a line ending uses `\n` or `\r\n`
- **THEN** the preprocessor preserves that line-ending style for the corresponding output line

### Requirement: Directive-Specific Regression Coverage
The project MUST include directive-focused tests that verify handler behavior and line-mapping guarantees for the preprocessing pipeline.

#### Scenario: Pipeline and require tests run in CI
- **WHEN** the Rust test suite executes for this project
- **THEN** directive-specific preprocessing tests validate `require` handler behavior and line-mapping invariants
