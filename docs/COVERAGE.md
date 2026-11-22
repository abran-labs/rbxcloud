# Open Cloud coverage

Generated from `openapi/coverage.json`. Edit the manifest, not this summary.

## Baseline

- Spec snapshot: `openapi/roblox.json` (OpenAPI 3.0.4)
- Upstream: <https://create.roblox.com/docs/cloud/openapi.json>
- SHA-256: `c124c92cf75500f1503c553422bdda38a5bc94d5c1acc33caeffc3d10985d88d`

## Totals

| Set | Operations |
|---|---:|
| Operations in the spec | 997 |
| Accept an API key (coverage target) | 397 |
| Existing typed method and CLI command | 46 |
| Pending | 351 |
| Excluded (no API-key auth) | 600 |

An operation is in the target when its `security` list contains exactly `{"roblox-api-key": []}` as one alternative. Operations that also accept OAuth 2.0 or a legacy cookie stay in the target; operations that never accept an API key are listed under `excluded` in the manifest and are never counted as required.

| Exclusion reason | Operations |
|---|---:|
| `legacy-cookie` | 524 |
| `no-api-key` | 76 |

## Families

`family` is the typed client module that owns the operation. `cli` is the explicit command that owns it: the real command for `existing` records, the planned command for `pending` ones. No record is owned by a generic request command.

| Family | CLI group | Existing | Pending | Total |
|---|---|---:|---:|---:|
| `advertising` | `advertising` | 0 | 22 | 22 |
| `analytics_alerts` | `analytics-alerts` | 0 | 5 | 5 |
| `analytics_dashboards` | `analytics-dashboards` | 0 | 9 | 9 |
| `analytics_logs` | `analytics-logs` | 0 | 7 | 7 |
| `analytics_queries` | `analytics-queries` | 0 | 5 | 5 |
| `asset_delivery` | `asset-delivery` | 0 | 2 | 2 |
| `asset_permissions` | `asset-permissions` | 0 | 1 | 1 |
| `assets` | `assets` | 6 | 9 | 15 |
| `avatar_bundles` | `avatar-bundles` | 0 | 5 | 5 |
| `badges` | `badges` | 0 | 3 | 3 |
| `collectibles` | `collectibles` | 0 | 20 | 20 |
| `configs` | `configs` | 0 | 9 | 9 |
| `creator_store` | `creator-store` | 0 | 10 | 10 |
| `data_stores` | `datastore-v2` | 0 | 18 | 18 |
| `data_stores_v1` | `datastore` | 8 | 0 | 8 |
| `developer_products` | `developer-products` | 0 | 4 | 4 |
| `experience_notifications` | `experience-notifications` | 0 | 8 | 8 |
| `experiments` | `experiments` | 0 | 11 | 11 |
| `game_events` | `game-events` | 0 | 5 | 5 |
| `game_localization` | `game-localization` | 0 | 40 | 40 |
| `game_passes` | `game-passes` | 0 | 4 | 4 |
| `generative_ai` | `generative-ai` | 0 | 2 | 2 |
| `groups` | `group` | 3 | 19 | 22 |
| `inventory` | `inventory` | 1 | 0 | 1 |
| `item_folders` | `item-folders` | 0 | 8 | 8 |
| `localization_tables` | `localization-tables` | 0 | 10 | 10 |
| `luau_execution` | `luau` | 4 | 1 | 5 |
| `marketplace_items` | `marketplace-items` | 0 | 13 | 13 |
| `matchmaking` | `matchmaking` | 0 | 23 | 23 |
| `memory_stores` | `memory-store` | 0 | 10 | 10 |
| `messaging` | `messaging` | 1 | 1 | 2 |
| `notifications` | `notification` | 1 | 0 | 1 |
| `ordered_data_stores` | `ordered-datastore-v2` | 0 | 6 | 6 |
| `ordered_data_stores_v1` | `ordered-datastore` | 6 | 0 | 6 |
| `place_instances` | `place-instances` | 0 | 4 | 4 |
| `place_versions` | `place-versions` | 0 | 3 | 3 |
| `places` | `experience`, `place` | 3 | 0 | 3 |
| `secrets` | `secrets` | 0 | 5 | 5 |
| `server_management` | `server-management` | 0 | 7 | 7 |
| `service_efficiency` | `service-efficiency` | 0 | 23 | 23 |
| `subscriptions` | `subscription` | 1 | 0 | 1 |
| `team_create` | `team-create` | 0 | 6 | 6 |
| `thumbnails` | `thumbnails` | 0 | 3 | 3 |
| `universe_followings` | `universe-followings` | 0 | 5 | 5 |
| `universes` | `universe` | 3 | 4 | 7 |
| `user_restrictions` | `user-restriction` | 7 | 0 | 7 |
| `users` | `user` | 2 | 1 | 3 |
| **Total** | | **46** | **351** | **397** |

## Servers

Most operations use the spec default server. These do not, so a client that only knows the default base URL cannot reach them:

| Server | Operations |
|---|---:|
| `https://apis.roblox.com` | 348 |
| `https://itemconfiguration.roblox.com` | 46 |
| `https://thumbnails.roblox.com` | 3 |

## Existing commands outside the spec

These commands send requests that have no API-key operation in this spec snapshot. They are kept for compatibility and are not counted above.

| Command | Request |
|---|---|
| `rbxcloud group shout` | `GET /cloud/v2/groups/{group_id}/shout` |
| `rbxcloud luau get-task (without --version-id)` | `GET /cloud/v2/universes/{universe_id}/places/{place_id}/luau-execution-sessions/{session_id}/tasks/{task_id}` |
| `rbxcloud luau get-logs (without --version-id)` | `GET /cloud/v2/universes/{universe_id}/places/{place_id}/luau-execution-sessions/{session_id}/tasks/{task_id}/logs` |
| `rbxcloud user-restriction logs (with --place-id)` | `GET /cloud/v2/universes/{universe_id}/places/{place_id}/user-restrictions:listLogs` |

## Notes

- `GET /cloud/v2/universes/{universe_id}/subscription-products/{subscription_product_id}/subscriptions/{subscription_id}`: Known defect: `subscription get` (src/rbx/v2/subscription.rs) owns this operation but requests .../subscription-products/{product} without /subscriptions/{subscription_id}, so it does not reach the documented path. Fix when the subscriptions family is migrated.

## Manifest schema

`openapi/coverage.json` has `schema_version`, `spec`, `spec_sha256`, `summary`, `operations` and `excluded`.

| Field | Meaning |
|---|---|
| `id` | Stable key, `HTTP_METHOD /path` exactly as in the spec |
| `method`, `path` | The two halves of `id` |
| `auth` | Accepted schemes: `api-key`, `oauth2`, `legacy-cookie`, `none` |
| `family` | Typed client module that owns the operation |
| `cli` | Owning command, `rbxcloud <group> <command>` |
| `cli_aliases` | Other existing commands that call the same operation |
| `status` | `existing` or `pending` |
| `operation_id`, `tags`, `server`, `stability` | Copied from the spec |
| `note` | Known mismatch between the owner and the spec |

`excluded` records carry `id`, `method`, `path`, `auth` and `reason` only.

`cargo test --test coverage_manifest` checks the manifest against the spec snapshot.
