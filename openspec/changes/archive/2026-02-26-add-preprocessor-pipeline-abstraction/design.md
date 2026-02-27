## Context

`src/preprocessor.rs` currently implements directive handling inline, centered on `require` rewriting and temporary-file generation. The implementation preserves line mapping by replacing recognized directive lines with commented lines, which is critical for accurate parse and mismatch diagnostics. The current structure works for one directive but does not scale cleanly to additional directives because parsing, transform policy, and output writing are tightly coupled.

This change needs to improve extensibility without changing current CLI-visible behavior, especially around `require` handling and line-location fidelity.

## Goals / Non-Goals

**Goals:**
- Introduce a composable preprocessor pipeline with ordered directive handlers.
- Keep current `require` behavior by implementing it as a dedicated handler in the new pipeline.
- Preserve deterministic line mapping and newline-style preservation guarantees.
- Add directive-specific tests that lock behavior for handler output and mapping invariants.

**Non-Goals:**
- Add new end-user directives in this change.
- Change extension loading policy semantics (`require` remains best-effort load behavior at runtime).
- Alter runner orchestration or failure report rendering outside what is required for preprocessor extraction.

## Decisions

1. **Model preprocessing as a pipeline of handlers evaluated in order**
   - Introduce an internal handler interface that receives line context and can emit a transformed line and directive side effects.
   - Run handlers in deterministic order with first-match-wins semantics for directive ownership.
   - **Alternative considered:** global match-and-dispatch map only. Rejected because deterministic ordering and overlap policy are clearer with an explicit ordered pipeline.

2. **Represent `require` as a concrete handler with parity semantics**
   - Extract existing `require` parsing and directive-collection logic into `RequireDirectiveHandler`.
   - Preserve current transformation approach (`# <original line>`), whitespace/newline handling, and extension token extraction behavior.
   - **Alternative considered:** parse all directives into an AST before transform. Rejected as unnecessary complexity for current scope and would increase migration risk.

3. **Keep line-mapping guarantees as an explicit invariant**
   - Pipeline emits exactly one output line per input line and preserves line ending style.
   - Non-directive and comment lines pass through unchanged.
   - **Alternative considered:** allow handlers to insert/remove lines. Rejected because it breaks current failure-location guarantees and would require a full source-map mechanism.

4. **Add directive-focused tests at two levels**
   - Unit-style tests for handler behavior (`require` parsing/rewriting and edge cases).
   - End-to-end preprocessor tests validating line-count and line-content mapping, plus existing CLI integration regressions.
   - **Alternative considered:** rely only on existing CLI tests. Rejected because they do not sufficiently isolate preprocessor invariants.

## Risks / Trade-offs

- **[Risk] Handler abstraction introduces extra indirection** -> **Mitigation:** keep interface minimal and colocate default handler registrations near preprocessing entrypoint.
- **[Risk] Subtle parity drift in `require` token parsing** -> **Mitigation:** port existing behavior first, then add golden tests for known edge cases.
- **[Risk] Future handlers may require richer transforms than one-line-in/one-line-out** -> **Mitigation:** document current invariant and treat multi-line transforms as a separate, explicit future change.

## Migration Plan

- Refactor in-place behind the existing `preprocess_file` public entrypoint so callers remain unchanged.
- Introduce pipeline and `require` handler with behavior parity tests before cleanup.
- Run existing integration tests to confirm no user-visible regressions.
- Rollback strategy: revert the refactor commit if parity checks fail.

## Open Questions

- Should unknown directive-like lines be observable through optional debug logging in a future change, or remain silent pass-through only?
