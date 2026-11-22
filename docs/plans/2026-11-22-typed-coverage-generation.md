# Typed Open Cloud coverage generation plan

**Goal:** Turn all 397 official API-key operations into explicit typed Rust SDK methods and Clap commands. No catch-all.

**Architecture:** Deterministic stdlib Python maintainer generator consumes pinned spec, emits reviewable Rust models and named operation/Clap structs grouped by official tag/service. Shared authenticated transport, explicit methods/paths and typed parameters. Manual multipart/legacy modules. Generated files checked in, reproducibility tested. Typed structs/enums for schemas; dynamic values only where schema defines one. Never arbitrary user endpoint/method/body JSON.

**Global constraints:** Preserve old CLI/API. No secrets argv. No live tests. No generic api/request/raw/call. Explicit command and typed SDK method per operation. Manifest `implemented` only with command/method/path/params contract tests. Off-default hosts supported. File modules grouped and bounded. `list` vs `get` explicit.

## Tasks
1. `scripts/gen_typed_api.py`, `tools/typed_api/*`, golden fixtures. Test ref/requestBody resolution, primitives/arrays/maps/objects/enums/compositions, exact JSON names, Rust/CLI identifier collisions, list vs get, no generic passthrough, byte-stable generation. Resolve ambiguity with explicit registry, no silent collapse.
2. Shared secure client transport from foundation Task 2; test local server, credentials, redirects, retries, bounded operations, multipart.
3. Data/memory stores: every Data and memory stores operation (v2 data stores, entries/scopes/revisions/snapshots, ordered, queues, sorted maps, operations/flush); explicit groups datastore-v2, ordered-datastore-v2, memory-store. Test all paths/queries/JSON/pagination/errors.
4. Assets/creators/experiences: every Assets, Groups, Users, Universes, Places, Instances, Luau, Subscriptions, Notifications, Creator Store operation. Typed multipart metadata, Image+Decal, polling, update masks, host selection, secret redaction; fix legacy subscription path.
5. Economy/config/live ops: Developer products, Game passes, Configs, Experiments, Matchmaking, Server management, Virtual events, Team Create, Localization, Badges. Typed JSON, safe multipart, explicit destructive commands, contract tests.
6. Advertising, Analytics, Service efficiency, Item configuration, Thumbnails and remaining families. Explicit flags, repeated arrays, nondefault hosts, typed responses. Test every family.
7. Audit reproducibility, compile, Clippy/tests, all 397 accounted, no generic command, help per owner. Scope availability isn't proof. Migrate roblox-ads only after equivalence; restore short roblox-open-cloud skill.
