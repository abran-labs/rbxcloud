# Open Cloud coverage

Generated from `openapi/coverage.json` by `python3 scripts/gen_coverage.py`. Do not edit by hand: edit the manifest, then run the script.

## Baseline

- Spec snapshot: `openapi/roblox.json` (OpenAPI 3.0.4)
- Upstream: <https://create.roblox.com/docs/cloud/openapi.json>
- SHA-256: `c124c92cf75500f1503c553422bdda38a5bc94d5c1acc33caeffc3d10985d88d`

## Totals

| Set | Operations |
|---|---:|
| Operations in the spec | 997 |
| Accept an API key (coverage target) | 397 |
| Covered (`existing` + `implemented`) | 335 |
| Not covered (`defective` + `pending`) | 62 |
| Excluded (no API-key auth) | 600 |

An operation is in the target when its `security` list contains exactly `{"roblox-api-key": []}` as one alternative. Operations that also accept OAuth 2.0 or a legacy cookie stay in the target; operations that never accept an API key are listed under `excluded` in the manifest and are never counted as required.

| Exclusion reason | Operations |
|---|---:|
| `legacy-cookie` | 524 |
| `no-api-key` | 76 |

## Statuses

| Status | Operations | Covered | Meaning |
|---|---:|---|---|
| `existing` | 45 | yes | A legacy client (`src/rbx/v1`, `src/rbx/v2`) and its command reach this method and path. Request parameters may be incomplete; see Notes. No new work may use this status. |
| `implemented` | 290 | yes | A typed method on the shared transport plus an explicit command. Every newly built or migrated operation gets this status. |
| `defective` | 0 | no | A legacy command targets this operation but does not send the documented request. Needs a `note`. |
| `pending` | 62 | no | No owner code yet. `cli` is the planned command and may still be renamed. |

## Families

`family` is the typed client module that owns the operation. `cli` is the explicit command that owns it: the real command for `existing`, `implemented` and `defective` records, the planned command for `pending` ones. No record is owned by a generic request command.

| Family | CLI group | Existing | Implemented | Defective | Pending | Total |
|---|---|---:|---:|---:|---:|---:|
| `advertising` | `action-tools`, `advertising`, `campaign-planning`, `insights-tools`, `utility-tools` | 0 | 17 | 0 | 0 | 5 | 22 |
| `analytics_alerts` | `analytics-alerts`, `insights-tools` | 0 | 3 | 0 | 0 | 2 | 5 |
| `analytics_dashboards` | `action-tools`, `analytics-dashboards`, `insights-tools` | 0 | 5 | 0 | 0 | 4 | 9 |
| `analytics_logs` | `analytics-logs`, `composed-tools`, `insights-tools` | 0 | 5 | 0 | 0 | 2 | 7 |
| `analytics_queries` | `analytics-queries`, `insights-tools` | 0 | 2 | 0 | 0 | 3 | 5 |
| `asset_delivery` | `asset-download` | 0 | 2 | 0 | 0 | 0 | 2 |
| `asset_permissions` | `asset-permissions` | 0 | 0 | 0 | 0 | 1 | 1 |
| `assets` | `asset-multipart`, `asset-multipart-start`, `asset-tools`, `assets` | 6 | 9 | 0 | 0 | 0 | 15 |
| `avatar_bundles` | `action-tools`, `avatar-bundles`, `resource-tools`, `utility-tools` | 0 | 4 | 0 | 0 | 1 | 5 |
| `badges` | `action-tools`, `badges` | 0 | 1 | 0 | 0 | 2 | 3 |
| `collectibles` | `action-tools`, `collectibles`, `resource-tools` | 0 | 17 | 0 | 0 | 3 | 20 |
| `configs` | `composed-tools`, `configs`, `liveops-tools` | 0 | 6 | 0 | 0 | 3 | 9 |
| `creator_store` | `creator-store`, `resource-tools`, `utility-tools` | 0 | 6 | 0 | 0 | 4 | 10 |
| `data_stores` | `datastore-v2` | 0 | 18 | 0 | 0 | 0 | 18 |
| `data_stores_v1` | `datastore` | 8 | 0 | 0 | 0 | 0 | 8 |
| `developer_products` | `developer-products`, `liveops-tools` | 0 | 2 | 0 | 0 | 2 | 4 |
| `experience_notifications` | `composed-tools`, `resource-tools` | 0 | 8 | 0 | 0 | 0 | 8 |
| `experiments` | `composed-tools`, `experiments`, `liveops-tools` | 0 | 8 | 0 | 0 | 3 | 11 |
| `game_events` | `game-events`, `liveops-tools` | 0 | 3 | 0 | 0 | 2 | 5 |
| `game_localization` | `action-tools`, `game-localization`, `resource-tools`, `utility-tools` | 0 | 33 | 0 | 0 | 7 | 40 |
| `game_passes` | `game-passes`, `liveops-tools` | 0 | 2 | 0 | 0 | 2 | 4 |
| `generative_ai` | `generative-ai`, `utility-tools` | 0 | 1 | 0 | 0 | 1 | 2 |
| `groups` | `creator-tools`, `group` | 3 | 19 | 0 | 0 | 0 | 22 |
| `inventory` | `inventory` | 1 | 0 | 0 | 0 | 0 | 1 |
| `item_folders` | `action-tools`, `item-folders`, `resource-tools` | 0 | 6 | 0 | 0 | 2 | 8 |
| `localization_tables` | `action-tools`, `localization-tables`, `resource-tools` | 0 | 8 | 0 | 0 | 2 | 10 |
| `luau_execution` | `creator-tools`, `luau` | 4 | 1 | 0 | 0 | 0 | 5 |
| `marketplace_items` | `resource-tools`, `utility-tools` | 0 | 13 | 0 | 0 | 0 | 13 |
| `matchmaking` | `action-tools`, `liveops-tools`, `matchmaking` | 0 | 15 | 0 | 0 | 8 | 23 |
| `memory_stores` | `memory-store` | 0 | 10 | 0 | 0 | 0 | 10 |
| `messaging` | `action-tools`, `messaging` | 1 | 1 | 0 | 0 | 0 | 2 |
| `notifications` | `notification` | 1 | 0 | 0 | 0 | 0 | 1 |
| `ordered_data_stores` | `ordered-datastore-v2` | 0 | 6 | 0 | 0 | 0 | 6 |
| `ordered_data_stores_v1` | `ordered-datastore` | 6 | 0 | 0 | 0 | 0 | 6 |
| `place_instances` | `creator-tools`, `place-instances` | 0 | 3 | 0 | 0 | 1 | 4 |
| `place_versions` | `composed-tools`, `resource-tools` | 0 | 3 | 0 | 0 | 0 | 3 |
| `places` | `experience`, `place` | 3 | 0 | 0 | 0 | 0 | 3 |
| `secrets` | `action-tools`, `resource-tools` | 0 | 5 | 0 | 0 | 0 | 5 |
| `server_management` | `liveops-tools`, `server-management` | 0 | 6 | 0 | 0 | 1 | 7 |
| `service_efficiency` | `resource-tools`, `service-efficiency`, `service-payments` | 0 | 22 | 0 | 0 | 1 | 23 |
| `subscriptions` | `creator-tools` | 0 | 1 | 0 | 0 | 0 | 1 |
| `team_create` | `action-tools`, `resource-tools` | 0 | 6 | 0 | 0 | 0 | 6 |
| `thumbnails` | `resource-tools` | 0 | 3 | 0 | 0 | 0 | 3 |
| `universe_followings` | `resource-tools` | 0 | 5 | 0 | 0 | 0 | 5 |
| `universes` | `creator-tools`, `universe` | 3 | 4 | 0 | 0 | 0 | 7 |
| `user_restrictions` | `user-restriction` | 7 | 0 | 0 | 0 | 0 | 7 |
| `users` | `creator-tools`, `user` | 2 | 1 | 0 | 0 | 0 | 3 |
| **Total** | | **45** | **290** | **0** | **0** | **62** | **397** |

## Servers

`id` is `HTTP_METHOD /path` and does not include the host. An endpoint is located by `server` plus `path`. Operations per server:

| Server | Operations |
|---|---:|
| `https://apis.roblox.com` | 348 |
| `https://itemconfiguration.roblox.com` | 46 |
| `https://thumbnails.roblox.com` | 3 |

## Legacy commands outside the spec

These commands send requests that have no API-key operation in this spec snapshot. They are kept for compatibility and are not counted above.

| Command | When | Request |
|---|---|---|
| `rbxcloud group shout` | always | `GET /cloud/v2/groups/{group_id}/shout` |
| `rbxcloud luau get-task` | without --version-id | `GET /cloud/v2/universes/{universe_id}/places/{place_id}/luau-execution-sessions/{session_id}/tasks/{task_id}` |
| `rbxcloud luau get-logs` | without --version-id | `GET /cloud/v2/universes/{universe_id}/places/{place_id}/luau-execution-sessions/{session_id}/tasks/{task_id}/logs` |
| `rbxcloud user-restriction logs` | with --place-id | `GET /cloud/v2/universes/{universe_id}/places/{place_id}/user-restrictions:listLogs` |

## Notes

Known mismatches between an owner and the spec. Legacy clients are not changed here; each mismatch is fixed when its family is migrated.

| Operation | Status | Command | Note |
|---|---|---|---|
| `PATCH /assets/v1/assets/{assetId}` | `existing` | `rbxcloud assets update` | Partial: the spec has an `updateMask` query parameter; the client never sends it. |
| `GET /cloud/v2/universes/{universe_id}/places/{place_id}/user-restrictions` | `existing` | `rbxcloud user-restriction list` | Partial: the client sends a `filter` query parameter that the spec does not list for this operation (spec: `maxPageSize`, `pageToken`). |
| `PATCH /cloud/v2/universes/{universe_id}/places/{place_id}/user-restrictions/{user_restriction_id}` | `existing` | `rbxcloud user-restriction update` | Partial: `updateMask` is hard-coded to `gameJoinRestriction`; the caller cannot choose the mask. |
| `GET /cloud/v2/universes/{universe_id}/subscription-products/{subscription_product_id}/subscriptions/{subscription_id}` | `implemented` | `rbxcloud creator-tools subscription get` | Defective: `subscription get` (src/rbx/v2/subscription.rs) requests .../subscription-products/{product} and never appends /subscriptions/{subscription_id}, so it does not reach this operation. |
| `GET /cloud/v2/universes/{universe_id}/user-restrictions` | `existing` | `rbxcloud user-restriction list` | Partial: the client sends a `filter` query parameter that the spec does not list for this operation (spec: `maxPageSize`, `pageToken`). |
| `PATCH /cloud/v2/universes/{universe_id}/user-restrictions/{user_restriction_id}` | `existing` | `rbxcloud user-restriction update` | Partial: `updateMask` is hard-coded to `gameJoinRestriction`; the caller cannot choose the mask. |
| `GET /datastores/v1/universes/{universeId}/standard-datastores/datastore/entries` | `existing` | `rbxcloud datastore list` | Partial: the client sends the query parameter as `AllScopes`; the spec name is `allScopes`. |
| `POST /universes/v1/{universeId}/places/{placeId}/versions` | `existing` | `rbxcloud experience publish` | Partial: the client sends only `versionType`; the spec also has `isOldVersionAllowed` and `shouldForceRestart`. |

## Manifest schema

`openapi/coverage.json` has `schema_version`, `spec`, `spec_sha256`, `summary`, `outside_spec`, `operations` and `excluded`.

| Field | Meaning |
|---|---|
| `id` | Stable key, `HTTP_METHOD /path` exactly as in the spec |
| `method`, `path` | The two halves of `id` |
| `auth` | Accepted schemes, see below |
| `family` | Typed client module that owns the operation |
| `cli` | Owning command, `rbxcloud <group> <command>` |
| `cli_aliases` | Other real commands that call the same operation |
| `status` | `existing`, `implemented`, `defective` or `pending` |
| `operation_id`, `tags`, `server`, `stability` | Copied from the spec |
| `note` | Known mismatch between the owner and the spec |

`excluded` records carry `id`, `method`, `path`, `auth` and `reason` only.

| `auth` value | Spec `security` |
|---|---|
| `api-key` | `roblox-api-key`, the `x-api-key` header |
| `oauth2` | `roblox-oauth2` |
| `legacy-cookie` | `roblox-legacy-cookie`, the `.ROBLOSECURITY` cookie |
| `none` | An empty requirement `{}`: callable without credentials |
| `unspecified` | The operation has no `security` key. Only on `excluded` records |

## Maintenance

- Change coverage: edit the record in `openapi/coverage.json` (`status`, `cli`, `cli_aliases`, `family`, `note`), then run `python3 scripts/gen_coverage.py` to refresh `summary` and this page.
- Refresh the spec: replace `openapi/roblox.json`, then run the script. Owner fields of operations already in the manifest are kept; new operations are seeded as `pending`.
- `cargo test --test coverage_manifest` checks the manifest against the spec snapshot and this page against the manifest.
