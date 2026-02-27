## Context

`duckdb-slt` currently validates behavior with focused local fixtures, and the directive preprocessor only supports `require`. Real DuckDB sqllogictest files use additional directives and edge-case record combinations that are not represented in the current fixture set. To prevent compatibility drift, we need a deterministic harness that runs a curated upstream corpus subset and a preprocessor design that can absorb more DuckDB directives without fragile, one-off parsing logic.

## Goals / Non-Goals

**Goals:**
- Run a pinned, curated subset of DuckDB's real sqllogictest corpus as part of automated compatibility validation.
- Expand directive compatibility for the selected subset by evolving preprocessing from single-directive handling into an extensible directive pipeline.
- Preserve stable diagnostics (especially line mapping and file references) when directives are rewritten or records are skipped.
- Keep harness execution deterministic and CI-friendly (no runtime network dependency).

**Non-Goals:**
- Full parity with every directive and workflow from DuckDB's entire sqllogictest corpus.
- Replacing the `sqllogictest` parser with a custom parser.
- Introducing online corpus fetches during normal test runs.

## Decisions

### 1) Add a pinned corpus-subset package in-repo
We will add a checked-in corpus subset with a manifest that records upstream source (repository + commit/version), selected files, and any local normalization notes. The harness will run only this pinned subset.

**Why:** Deterministic local/CI behavior, reproducible failures, and easy review of corpus updates.

**Alternatives considered:**
- Download full corpus at test runtime: rejected due to nondeterminism, network flakiness, and long CI runs.
- Continue with synthetic-only fixtures: rejected because it does not measure real DuckDB compatibility.

### 2) Add a dedicated compatibility harness test target
We will add an integration harness that executes the pinned subset through `duckdb-slt` and fails on compatibility regressions. The harness will emit per-file diagnostics and summary output aligned with existing CLI reporting.

**Why:** Reuses the real CLI execution path and validates end-to-end compatibility behavior instead of isolated internals.

**Alternatives considered:**
- Unit-test only directive transforms: rejected because it misses runtime interactions between preprocessing, parser, runner, and driver.
- External one-off script outside `cargo test`: rejected because it weakens CI enforcement and discoverability.

### 3) Refactor preprocessing into an extensible directive pipeline
`src/preprocessor.rs` will evolve from a hard-coded `require` branch into directive handlers that can:
- Parse directive lines and collect structured effects (for example, extension loads).
- Rewrite unsupported parser keywords into comments while preserving line count.
- Apply skip/execute behavior for additional compatibility directives needed by the corpus subset.

The runner will consume collected directive effects in a single place before file execution.

**Why:** Keeps DuckDB compatibility logic explicit, testable, and easy to extend as new directives appear in future subsets.

**Alternatives considered:**
- Add more ad hoc branches in the current loop: rejected due to maintenance risk and increasing coupling.
- Handle directives after sqllogictest parsing: rejected because unsupported keywords must be normalized before parser ingestion.

### 4) Validate with directive-focused fixture tests plus corpus harness coverage
We will add focused fixtures for each newly supported directive behavior and keep end-to-end corpus subset runs as regression coverage.

**Why:** Unit/integration split makes failures easier to debug while preserving high-confidence compatibility checks.

## Risks / Trade-offs

- [Subset misses an important upstream directive pattern] -> Mitigation: require manifest metadata for covered directives and add/update subset selection criteria.
- [Line-number drift introduced by preprocessing rewrites] -> Mitigation: preserve exact newline structure and add tests asserting reported line locations on failure.
- [Directive semantics diverge from DuckDB expectations] -> Mitigation: prioritize directives observed in the real subset and add explicit fixture cases mirroring upstream patterns.
- [Harness increases test runtime] -> Mitigation: keep subset intentionally small and representative; monitor runtime budget in CI.

## Migration Plan

1. Land corpus subset files + manifest and add harness test entry point.
2. Refactor preprocessor into directive handlers while keeping existing `require` behavior green.
3. Add support for additional directives required by the selected subset and associated regression fixtures.
4. Enable harness in normal CI test workflow and document update procedure for refreshing the subset.

Rollback is straightforward: disable the harness test target and revert subset/directive additions without affecting baseline CLI operation.

## Open Questions

- Which exact directive set is required by the first selected corpus subset (for example `skipif`, `onlyif`, `mode`, `hash-threshold`), and which can be deferred?
- What is the target runtime budget for corpus compatibility tests in CI?
- Should subset refresh tooling be a Rust utility in-repo or a lightweight script under `scripts/`?
