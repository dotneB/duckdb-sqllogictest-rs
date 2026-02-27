## Why

The current preprocessor is single-purpose and tightly coupled to `require`, which makes future directive support harder to add and test safely. A composable pipeline abstraction is needed now to improve extensibility while preserving strict line-mapping behavior used for accurate failure locations.

## What Changes

- Introduce a composable preprocessor pipeline API that applies directives through discrete handlers.
- Keep `require` as a first-class directive handler with behavior parity to the current implementation.
- Preserve line-count and source-location guarantees so parser and mismatch diagnostics still point to correct original lines.
- Add directive-specific test suites to validate handler behavior and line-mapping invariants.
- Keep existing CLI-visible behavior unchanged for scripts that currently rely on `require` processing.

## Capabilities

### New Capabilities
- `preprocessor-pipeline`: Extensible directive-processing pipeline with deterministic transforms and stable source-line mapping.

### Modified Capabilities
- *(none)*

## Impact

- Affected code: `src/preprocessor.rs` (refactor into pipeline + handlers), possibly new supporting preprocessor module files.
- Affected tests: new directive-focused tests and existing `require` integration tests in `tests/cli.rs`.
- APIs/dependencies: no new external dependencies expected; internal preprocessor interfaces will change.
