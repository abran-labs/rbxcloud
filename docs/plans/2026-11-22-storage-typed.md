# Storage typed API implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task.

**Goal:** Ship explicit typed SDK and CLI coverage for all 34 v2 storage operations: 18 DataStores, 6 ordered DataStores, and 10 Memory Store operations.

**Architecture:** Build a deterministic maintainer generator for explicit operation definitions and typed Rust models. Generated files are checked in. All calls use `rbx::client::AsyncClient`; operations have fixed methods, host and path templates. Clap exposes one explicit command per manifest owner and typed fields for documented parameters and JSON body properties. No arbitrary path/method/endpoint or catch-all JSON body argument.

**Tech stack:** Rust, serde, Clap, existing AsyncClient; Python 3 stdlib generator; local test HTTP server.

**Spec:** `/home/abran/.config/opencode/work-plans/rbxcloud-complete-coverage.md` and `openapi/coverage.json`.

## Global constraints

- No generic `api`/`request`/`raw` command or runtime endpoint dispatcher.
- Every command maps to one official operation and typed SDK function.
- Generated Rust structs preserve exact JSON names, request requiredness and known enum values. Dynamic Google protobuf values use `serde_json::Value`.
- Async CLI only: `AsyncClient` created from `Credentials::from_process` and default policy, with test policy allowed only in crate tests.
- Preserve legacy v1 CLI commands.
- Local HTTP contract tests only, no live calls or live data.
- Coverage `implemented` requires typed method, explicit command, and contract test for exact method/path/parameters.
- Run generator twice and `--check`; hand-edit generator input/overrides, not generated output.
- Generated files are grouped by domain/responsibility; if any module grows beyond manageable size split it.

## Task 1: Typed operation/model generator

Files: `scripts/gen_typed_api.py`, `tools/typed_api/*.py`, `tests/typed_generation.rs`, generated `src/rbx/cloud/{mod.rs,models/*.rs,storage/*.rs}`, `src/cli/typed/{mod.rs,storage.rs}`.

Define explicit operation specs for the 34 target IDs in a manifest/overrides file. Generate:
- Rust enums for string enum schemas; object structs for documented schemas with `#[serde(rename_all = "camelCase")]` or exact renames; optional fields `Option<T>`, required arrays `Vec<T>`, integer types per format, bytes/string as documented, unknown dynamic structures as `Value`.
- `*Params` structs with path/query fields and named typed body structs. Public typed async methods on a `CloudClient` wrapping `AsyncClient`, e.g. `list_data_stores(ListDataStoresParams) -> Result<ListDataStoresResponse, Error>`.
- One Clap subcommand per exact manifest `cli`. Path and query parameters become explicit long flags. JSON body object properties become explicit long flags (`--value-json` for dynamic JSON, repeated typed fields where required) and the command constructs the typed body; no arbitrary endpoint or raw request arguments.
- Shared CLI runner that creates `AsyncClient` and prints typed serialized response; empty result prints nothing.
- Unit/golden tests for generator naming, enum variants, field preservation, nested schemas, query arrays, requiredness, no duplicate operations, deterministic output, and no generic CLI.

Start with tests that fail due missing generator/output; implement, generate, `cargo test --test typed_generation`, `cargo check`. Commit `feat: generate typed storage models and commands`.

## Task 2: Storage contract tests and CLI wiring

Files: `tests/typed_storage.rs`, registration in `src/cli/mod.rs` and `src/rbx/mod.rs`, `openapi/coverage.json`, `docs/COVERAGE.md` regenerated with `scripts/gen_coverage.py`.

For each of 34 operation IDs, test fixed method, path, path encoding, query names and JSON body fields using loopback policy. At minimum test each SDK method and each Clap parser; shared logic may be table-driven. Cover empty responses, dynamic DataStore values, increment body, snapshot, undelete, queue read/discard/flush, sorted-map patch and ordered entries. Verify credentials and network are absent for `--help`; no real key needed. Verify no arbitrary `url`, `method`, `path` or `body` flag. Keep `datastore` and `ordered-datastore` legacy help intact.

Update only the 34 records to `implemented`, regenerate doc, and assert counts. Run `cargo test --test typed_storage`, full `cargo test`, generator `--check`, `cargo fmt --check`. Commit `feat: add typed v2 storage commands`.

## Task 3: Follow-up generator generalization

Files: generator registry/templates only; no extra APIs yet.

Refactor tag/family grouping so subsequent domain plans can supply additional operation IDs and output modules without modifying storage code or global registration in conflicting ways. Test selecting noncontiguous operations, handling operation-level `$ref` parameters/requestBodies, off-default hosts, header parameters and schema-less bodies. Keep generator deterministic. Commit `refactor: generalize typed API generator`.
