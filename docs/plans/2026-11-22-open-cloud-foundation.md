# Open Cloud foundation implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Establish a secure, testable Rust transport and coverage inventory on which explicit typed Roblox Open Cloud commands can be added.

**Architecture:** Preserve the existing CLI. A shared blocking client owns credentials, retries, request/response handling, operation polling, and multipart files; domain clients and Clap commands remain explicit. A checked-in coverage manifest records each official API-key operation and its future typed client/CLI owner.

**Tech Stack:** Rust 2021, reqwest 0.12 blocking multipart/rustls, serde, Clap derive, tempfile for tests.

**Spec:** `/home/abran/.config/opencode/work-plans/rbxcloud-complete-coverage.md`

## Global constraints

- Every supported operation gets a typed SDK method and explicit Clap command. Never add an arbitrary method/path/body passthrough.
- Preserve the existing public CLI and library methods.
- Secrets come from environment or the secure local credentials file, never argv or logs.
- Never mutate live Roblox resources for testing. Tests use a local HTTP server.
- Use the checked-in official spec snapshot, with upstream URL and SHA-256, as the coverage baseline.
- Work directly on `main` in the new private-use fork, at user instruction to start immediately; do not release or deploy.

## Review focus

- A secret in the environment must not be included in formatted errors or emitted output (Task 2).
- Redirects must not send the API key to another host (Task 2).
- Pagination limits and bounded operation polling must not loop forever (Task 2).
- Every official operation must have an owner, even if a later task implements it (Task 1).
- Existing command flags and client methods must keep working (Task 3).

---

### Task 1: Coverage inventory

**Files:**
- Create: `openapi/roblox.json`
- Create: `openapi/coverage.json`
- Create: `docs/COVERAGE.md`
- Test: `tests/coverage_manifest.rs`

**Interfaces:**
- Produces: `openapi/coverage.json` with `schema_version: 1`, `spec_sha256`, and `operations` records containing stable `id` (`HTTP_METHOD /path`), `method`, `path`, `auth`, `family`, `cli`, and `status`.
- Every record initially has `status: "pending"` unless an existing method owns it, then `status: "existing"`.

- [ ] **Step 1: Write the failing test.** Assert that the spec SHA-256 matches the file, operation IDs are unique, API-key-auth records are nonempty and all have non-generic CLI owners and typed-family assignments, and no legacy-cookie operation is counted as required. Include at least `GET /cloud/v2/universes/{universe_id}/data-stores` and `POST /assets/v1/assets`.
- [ ] **Step 2: Run `cargo test --test coverage_manifest`; expect missing-file failure.**
- [ ] **Step 3: Generate the manifest from the downloaded official spec using `python3` with only stdlib. Treat `security: [{"roblox-api-key":[]}]` as API-key eligible. Group families by official tag and path semantics rather than using the root `cloud` bucket. Do not commit generated Rust code.**
- [ ] **Step 4: Run `cargo test --test coverage_manifest`; expect PASS.**
- [ ] **Step 5: Commit `feat: add Open Cloud coverage inventory`.**

### Task 2: Shared transport

**Files:**
- Create: `src/rbx/client.rs`
- Create: `src/rbx/credentials.rs`
- Modify: `src/rbx/mod.rs`
- Modify: `Cargo.toml`
- Test: `tests/transport.rs`

**Interfaces:**
- `Credentials::load(env: &str, credentials_file: &Path) -> Result<SecretString, Error>` accepts `ROBLOX_OPEN_CLOUD_API_KEY`, then legacy `RBXCLOUD_API_KEY`; existing secure `roblox-open-cloud.env` parsing is separate from the secret value.
- `Client::new(credentials: Credentials, policy: ClientPolicy) -> Result<Client, Error>`; `ClientPolicy` owns base URL, timeout, max retries, max poll attempts, and poll delay. Test base URLs support HTTP; default is `https://apis.roblox.com`.
- `Client::execute(Request) -> Result<serde_json::Value, Error>` handles JSON/query requests, explicit multipart attachments, retries only for safe/idempotent operations or documented idempotency, and bounded asynchronous operation polling.
- Response/error output includes HTTP status and server error message but never credential values or signed query secrets.

- [ ] **Step 1: Write failing transport tests.** Cover env precedence, file fallback, missing keys, malformed files, 204 responses, server JSON errors, redirect stripping, bounded 429 retries, bounded operation polling, multipart names/bytes, and secret redaction. Use a local std::net::TcpListener server rather than an external network.
- [ ] **Step 2: Run `cargo test --test transport`; expect missing-module failure.**
- [ ] **Step 3: Implement the interfaces and reqwest blocking client.** Keep secrets in a redacted wrapper; reject redirects by default. Use explicit request constructors; do not expose a CLI or arbitrary-string method/path/body constructor. Add focused modules if a file exceeds roughly 250 meaningful lines.
- [ ] **Step 4: Run `cargo test --test transport`; expect PASS.**
- [ ] **Step 5: Commit `feat: add shared Open Cloud transport`.**

### Task 3: Public CLI wiring and baseline compatibility

**Files:**
- Create: `src/cli/cloud.rs`
- Modify: `src/cli/mod.rs`
- Modify: `src/main.rs`
- Test: `tests/cli_baseline.rs`

**Interfaces:**
- Add `rbxcloud coverage` to report implemented/pending totals by family; no request command.
- Existing command construction stays intact; new explicit family commands consume the shared client in later plans.

- [ ] **Step 1: Write failing tests for `rbxcloud --help`, `coverage`, `assets create --help`, and `datastore --help`. Assert old flags are still shown and coverage does not expose request execution.**
- [ ] **Step 2: Run `cargo test --test cli_baseline`; expect failure.**
- [ ] **Step 3: Implement the coverage command and leave legacy wiring unchanged.**
- [ ] **Step 4: Run `cargo test --test cli_baseline` and `cargo test`; expect PASS.**
- [ ] **Step 5: Commit `feat: expose Open Cloud coverage status`.**

## Follow-on domain plans

Each plan starts from the manifest and adds explicit client methods/Clap subcommands plus contract tests for a coherent set of official tags: storage, assets, creators/games, economy/live-ops, advertising/analytics, remaining families, then migration and audit. No plan implements a catch-all. API families are parallel work after Tasks 1-3.
