## 1. Implementation
- [x] 1.1 Add a small parser/preprocessor for `require` directives (extract required extension specs and produce preprocessed script content).
- [x] 1.2 Integrate `require` handling into per-file execution (attempt `LOAD` for required extensions before running the file; ignore load failures).
- [x] 1.3 Ensure preprocessing does not shift line numbers and user-facing diagnostics map back to the original input file path.
- [x] 1.4 Add CLI integration tests + fixtures that include `require` lines.
- [x] 1.5 Update `README.md` to document `require` support and its semantics.

## 2. Validation
- [x] 2.1 `cargo fmt --check`
- [x] 2.2 `cargo clippy -- -D warnings`
- [x] 2.3 `cargo test`
