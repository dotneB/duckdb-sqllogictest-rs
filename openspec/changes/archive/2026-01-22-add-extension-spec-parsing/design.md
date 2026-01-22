## Context
The CLI accepts repeatable `--extensions <EXT>` flags. DuckDB supports `INSTALL` and `LOAD` with either an extension name or a `.duckdb_extension` file path, and supports installing from different repositories via `INSTALL <name> FROM <repo>` (including custom repository URLs).

This change defines a small extension-spec grammar and maps it to concrete `INSTALL`/`LOAD` SQL.

## Goals / Non-Goals
- Goals:
  - Accept a small, explicit set of extension spec forms.
  - Produce safe, deterministic SQL for `INSTALL` and `LOAD`.
  - Apply extensions before running tests.
- Non-Goals:
  - Supporting a “load only” or “install only” mode.

## Decisions
- Decision: Parse `--extensions` entries into one of:
  - `Name("json")`
  - `RepositoryName("spatial", "community")` (from `spatial@community`)
  - `RepositoryName("httpfs", "core_nightly")` (from `httpfs@core_nightly`)
  - `RepositoryName("custom_extension", "https://my-custom-extension-repository")` (from `custom_extension@https://my-custom-extension-repository`)
  - `Path("/abs/or/rel/foo.duckdb_extension")`
- Decision: Treat an extension spec as a path when it either:
  - ends with `.duckdb_extension`, or
  - contains a path separator (`/` or `\\`).
- Decision: If an extension spec contains an `@`, treat it as `name@repository` (even if the repository contains `/`).
- Decision: Resolve relative path specs against `--workdir` (when provided) before generating SQL.
- Decision: Generate SQL using SQL string literals and escape single quotes by doubling them (`'` -> `''`).
  - For `Name(n)`: `INSTALL '<n>';` then `LOAD '<n>';`
  - For `RepositoryName(n, r)`: `INSTALL '<n>' FROM <r>;` then `LOAD '<n>';`
  - For `Path(p)`: `INSTALL '<p>';` then `LOAD '<p>';`

## Risks / Trade-offs
- Some strings that contain a slash but are not intended as paths will be treated as paths (unless they are used as the repository portion of `name@repository`).
- Relying on string literals for extension names is slightly less idiomatic than identifiers, but simplifies safe escaping and keeps one codepath.
