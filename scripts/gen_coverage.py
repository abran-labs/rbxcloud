#!/usr/bin/env python3
"""Maintain openapi/coverage.json and docs/COVERAGE.md. Stdlib only.

Run from the repo root:

    python3 scripts/gen_coverage.py            # merge spec + manifest, rewrite both files
    python3 scripts/gen_coverage.py --check    # exit 1 if either file would change
    python3 scripts/gen_coverage.py --reseed   # ignore the manifest, rebuild from the tables below
    python3 scripts/gen_coverage.py --list     # print status, family, cli, id per operation

The manifest is the source of truth for ownership. For every operation id that
is already in openapi/coverage.json, `family`, `cli`, `cli_aliases`, `status`
and `note` are kept exactly as they are. The tables in this file only seed
operations that are new in the spec snapshot. Spec-derived fields (`auth`,
`operation_id`, `tags`, `server`, `stability`), `spec_sha256`, `summary` and
docs/COVERAGE.md are always recomputed.

To change coverage: edit the record in openapi/coverage.json, then run this
script to refresh `summary` and docs/COVERAGE.md.

To refresh the spec: replace openapi/roblox.json, then run this script. New
operations get a seeded owner (or the script stops and asks for a family rule or
command name); operations that left the spec are reported on stderr.
"""
import collections
import hashlib
import json
import re
import os
import sys

SPEC_PATH = "openapi/roblox.json"
MANIFEST_PATH = "openapi/coverage.json"
DOC_PATH = "docs/COVERAGE.md"
SOURCE_URL = "https://create.roblox.com/docs/cloud/openapi.json"
DEFAULT_SERVER = "https://apis.roblox.com"
METHODS = ("get", "post", "put", "patch", "delete", "head", "options", "trace")
SCHEME = {
    "roblox-api-key": "api-key",
    "roblox-oauth2": "oauth2",
    "roblox-legacy-cookie": "legacy-cookie",
}
AUTH_ORDER = ["api-key", "oauth2", "legacy-cookie", "none"]

# existing:    a client that predates the shared transport reaches this operation.
# implemented: a typed method on the shared transport plus an explicit CLI command.
# defective:   a legacy command targets this operation but does not send the
#              documented request. Counted as not covered.
# pending:     no owner code yet; `cli` is the planned command.
STATUSES = ["existing", "implemented", "unverified", "defective", "pending"]
COVERED = {"existing", "implemented"}
OWNER_FIELDS = ("family", "cli", "cli_aliases", "status", "note")

# (path prefix regex, family, cli group). First match wins.
FAMILIES = [
    (r"^/assets/v1/", "assets", "assets"),
    (r"^/cloud/v2/users/\{user_id\}/asset-quotas$", "assets", "assets"),
    (r"^/asset-delivery-api/", "asset_delivery", "asset-delivery"),
    (r"^/asset-permissions-api/", "asset_permissions", "asset-permissions"),
    (r"^/v1/collectibles", "collectibles", "collectibles"),
    (r"^/v1/(bundles|avatar-item-updates)", "avatar_bundles", "avatar-bundles"),
    (r"^/v1/folders", "item_folders", "item-folders"),
    (r"^/v1/(items|creations|delist-item|permissions|asset-types)", "marketplace_items", "marketplace-items"),
    (r"^/v1/(assets-thumbnail-3d|users/avatar-3d|users/outfit-3d)", "thumbnails", "thumbnails"),
    (r"^/cloud/v2/universes/\{universe_id\}/data-stores", "data_stores", "datastore-v2"),
    (r"^/cloud/v2/universes/\{universe_id\}/ordered-data-stores", "ordered_data_stores", "ordered-datastore-v2"),
    (r"^/cloud/v2/universes/\{universe_id\}/memory-store", "memory_stores", "memory-store"),
    (r"^/datastores/v1/", "data_stores_v1", "datastore"),
    (r"^/ordered-data-stores/v1/", "ordered_data_stores_v1", "ordered-datastore"),
    (r"^/cloud/v2/groups/", "groups", "group"),
    (r"^/legacy-groups/", "groups", "group"),
    (r"^/legacy-develop/v1/user/groups/", "groups", "group"),
    (r"^/cloud/v2/users/\{user_id\}/inventory-items", "inventory", "inventory"),
    (r"^/cloud/v2/users/\{user_id\}/notifications", "notifications", "notification"),
    (r"^/notifications/v1/", "experience_notifications", "experience-notifications"),
    (r"^/cloud/v2/users/", "users", "user"),
    (r"^/cloud/v2/universes/\{universe_id\}/subscription-products/", "subscriptions", "subscription"),
    (r"^/cloud/v2/universes/\{universe_id\}(/places/\{place_id\})?/user-restrictions", "user_restrictions", "user-restriction"),
    (r"^/cloud/v2/universes/\{universe_id\}/.*luau-execution", "luau_execution", "luau"),
    (r"^/cloud/v2/universes/\{universe_id\}/places/\{place_id\}/instances/", "place_instances", "place-instances"),
    (r"^/cloud/v2/universes/\{universeId\}/places/", "places", "place"),
    (r"^/universes/v1/\{universeId\}/places/", "places", "place"),
    (r"^/place-version-history-api/", "place_versions", "place-versions"),
    (r"^/cloud/v2/universes/\{universeId\}/secrets", "secrets", "secrets"),
    (r"^/cloud/v2/universes/\{universe_id\}:publishMessage$", "messaging", "messaging"),
    (r"^/messaging-service/", "messaging", "messaging"),
    (r"^/cloud/v2/universes/\{universe_id\}:(generateSpeechAsset|translateText)$", "generative_ai", "generative-ai"),
    (r"^/cloud/v2/universes/\{universeId\}(:restartServers)?$", "universes", "universe"),
    (r"^/legacy-develop/.*(teamcreate|teamtest)", "team_create", "team-create"),
    (r"^/legacy-develop/v1/universes/", "universes", "universe"),
    (r"^/legacy-followings/", "universe_followings", "universe-followings"),
    (r"^/cloud/v2/creator-store-products", "creator_store", "creator-store"),
    (r"^/toolbox-service/", "creator_store", "creator-store"),
    (r"^/developer-products/", "developer_products", "developer-products"),
    (r"^/game-passes/", "game_passes", "game-passes"),
    (r"^/legacy-badges/", "badges", "badges"),
    (r"^/legacy-publish/v1/badges/", "badges", "badges"),
    (r"^/legacy-game-internationalization/", "game_localization", "game-localization"),
    (r"^/legacy-localization-tables/", "localization_tables", "localization-tables"),
    (r"^/creator-configs-public-api/v1/configs/", "configs", "configs"),
    (r"^/creator-configs-public-api/v1/experimentation/", "experiments", "experiments"),
    (r"^/matchmaking-api/", "matchmaking", "matchmaking"),
    (r"^/server-management/", "server_management", "server-management"),
    (r"^/virtual-events/", "game_events", "game-events"),
    (r"^/ads-management/", "advertising", "advertising"),
    (r"^/analytics-alert-control-plane/", "analytics_alerts", "analytics-alerts"),
    (r"^/analytics-custom-dashboards/", "analytics_dashboards", "analytics-dashboards"),
    (r"^/analytics-(query-api|benchmark)/", "analytics_queries", "analytics-queries"),
    (r"^/(log-attribute-http-service|universe-performance-raqi)/", "analytics_logs", "analytics-logs"),
    (r"^/service-efficiency-api/", "service_efficiency", "service-efficiency"),
]

# Spec operations already served by a typed client method and CLI command in
# this repo: id -> (cli, [other commands hitting the same operation]).
D1 = "/datastores/v1/universes/{universeId}/standard-datastores"
O1 = "/ordered-data-stores/v1/universes/{universeId}/orderedDataStores/{orderedDataStore}/scopes/{scope}/entries"
U2 = "/cloud/v2/universes/{universe_id}"
EXISTING = {
    "POST /assets/v1/assets": ("assets create", []),
    "PATCH /assets/v1/assets/{assetId}": ("assets update", []),
    "GET /assets/v1/assets/{assetId}": ("assets get", []),
    "POST /assets/v1/assets/{assetId}:archive": ("assets archive", []),
    "POST /assets/v1/assets/{assetId}:restore": ("assets restore", []),
    "GET /assets/v1/operations/{operationId}": ("assets get-operation", []),
    f"GET {D1}": ("datastore list-stores", []),
    f"GET {D1}/datastore/entries": ("datastore list", []),
    f"GET {D1}/datastore/entries/entry": ("datastore get", []),
    f"POST {D1}/datastore/entries/entry": ("datastore set", []),
    f"DELETE {D1}/datastore/entries/entry": ("datastore delete", []),
    f"POST {D1}/datastore/entries/entry/increment": ("datastore increment", []),
    f"GET {D1}/datastore/entries/entry/versions": ("datastore list-versions", []),
    f"GET {D1}/datastore/entries/entry/versions/version": ("datastore get-version", []),
    f"GET {O1}": ("ordered-datastore list", []),
    f"POST {O1}": ("ordered-datastore create", []),
    f"GET {O1}/{{entry}}": ("ordered-datastore get", []),
    f"DELETE {O1}/{{entry}}": ("ordered-datastore delete", []),
    f"PATCH {O1}/{{entry}}": ("ordered-datastore update", []),
    f"POST {O1}/{{entry}}:increment": ("ordered-datastore increment", []),
    "POST /universes/v1/{universeId}/places/{placeId}/versions": ("experience publish", []),
    "POST /messaging-service/v1/universes/{universeId}/topics/{topic}": ("messaging publish", []),
    "GET /cloud/v2/groups/{group_id}": ("group get", []),
    "GET /cloud/v2/groups/{group_id}/roles": ("group roles", []),
    "GET /cloud/v2/groups/{group_id}/memberships": ("group memberships", []),
    "GET /cloud/v2/users/{user_id}/inventory-items": ("inventory list", []),
    f"POST {U2}/places/{{place_id}}/luau-execution-session-tasks": ("luau execute", []),
    f"POST {U2}/places/{{place_id}}/versions/{{version_id}}/luau-execution-session-tasks": ("luau execute", []),
    f"GET {U2}/places/{{place_id}}/versions/{{version_id}}/luau-execution-sessions/{{luau_execution_session_id}}/tasks/{{task_id}}": ("luau get-task", []),
    f"GET {U2}/places/{{place_id}}/versions/{{version_id}}/luau-execution-sessions/{{luau_execution_session_id}}/tasks/{{task_id}}/logs": ("luau get-logs", []),
    "POST /cloud/v2/users/{user_id}/notifications": ("notification send", []),
    "GET /cloud/v2/universes/{universeId}/places/{placeId}": ("place get", []),
    "PATCH /cloud/v2/universes/{universeId}/places/{placeId}": ("place update-name", ["place update-description", "place update-server-size"]),
    "GET /cloud/v2/universes/{universeId}": ("universe get", []),
    "PATCH /cloud/v2/universes/{universeId}": ("universe update-name", ["universe update-description"]),
    "POST /cloud/v2/universes/{universeId}:restartServers": ("universe restart", []),
    "GET /cloud/v2/users/{user_id}": ("user get", []),
    "GET /cloud/v2/users/{user_id}:generateThumbnail": ("user thumbnail", []),
    f"GET {U2}/user-restrictions": ("user-restriction list", []),
    f"GET {U2}/places/{{place_id}}/user-restrictions": ("user-restriction list", []),
    f"GET {U2}/user-restrictions/{{user_restriction_id}}": ("user-restriction get", []),
    f"GET {U2}/places/{{place_id}}/user-restrictions/{{user_restriction_id}}": ("user-restriction get", []),
    f"PATCH {U2}/user-restrictions/{{user_restriction_id}}": ("user-restriction update", []),
    f"PATCH {U2}/places/{{place_id}}/user-restrictions/{{user_restriction_id}}": ("user-restriction update", []),
    f"GET {U2}/user-restrictions:listLogs": ("user-restriction logs", []),
}

# Legacy commands aimed at a spec operation that do not send the documented request.
DEFECTIVE = {
    f"GET {U2}/subscription-products/{{subscription_product_id}}/subscriptions/{{subscription_id}}": "subscription get",
}

_UR_UPDATE = ("Partial: `updateMask` is hard-coded to `gameJoinRestriction`; the caller "
              "cannot choose the mask.")
_UR_LIST = ("Partial: the client sends a `filter` query parameter that the spec does not "
            "list for this operation (spec: `maxPageSize`, `pageToken`).")
NOTES = {
    f"GET {U2}/subscription-products/{{subscription_product_id}}/subscriptions/{{subscription_id}}":
        "Defective: `subscription get` (src/rbx/v2/subscription.rs) requests "
        ".../subscription-products/{product} and never appends /subscriptions/{subscription_id}, "
        "so it does not reach this operation.",
    "PATCH /assets/v1/assets/{assetId}":
        "Partial: the spec has an `updateMask` query parameter; the client never sends it.",
    "POST /universes/v1/{universeId}/places/{placeId}/versions":
        "Partial: the client sends only `versionType`; the spec also has "
        "`isOldVersionAllowed` and `shouldForceRestart`.",
    f"PATCH {U2}/user-restrictions/{{user_restriction_id}}": _UR_UPDATE,
    f"PATCH {U2}/places/{{place_id}}/user-restrictions/{{user_restriction_id}}": _UR_UPDATE,
    f"GET {D1}/datastore/entries":
        "Partial: the client sends the query parameter as `AllScopes`; the spec name is `allScopes`.",
    f"GET {U2}/user-restrictions": _UR_LIST,
    f"GET {U2}/places/{{place_id}}/user-restrictions": _UR_LIST,
}

# Legacy commands whose request has no API-key operation in the spec snapshot.
OUTSIDE_SPEC = [
    {"cli": "rbxcloud group shout", "when": None,
     "request": "GET /cloud/v2/groups/{group_id}/shout"},
    {"cli": "rbxcloud luau get-task", "when": "without --version-id",
     "request": "GET /cloud/v2/universes/{universe_id}/places/{place_id}/luau-execution-sessions/{session_id}/tasks/{task_id}"},
    {"cli": "rbxcloud luau get-logs", "when": "without --version-id",
     "request": "GET /cloud/v2/universes/{universe_id}/places/{place_id}/luau-execution-sessions/{session_id}/tasks/{task_id}/logs"},
    {"cli": "rbxcloud user-restriction logs", "when": "with --place-id",
     "request": "GET /cloud/v2/universes/{universe_id}/places/{place_id}/user-restrictions:listLogs"},
]

# Planned command names where derivation is ambiguous or unclear: id -> command.
DSV2 = f"{U2}/data-stores/{{data_store_id}}"
LOGRX = "/log-attribute-http-service/v1/universe-regexes"
DASH = "/analytics-custom-dashboards/v1/universes/{universeId}/custom-dashboards"
ALERT = "/analytics-alert-control-plane/v1/resource/{resourceType}/id/{resourceId}/alerts"
AQ = "/analytics-query-api/v1/universes/{universeId}"
GI = "/legacy-game-internationalization/v1"
LT = "/legacy-localization-tables/v1"
MM = "/matchmaking-api/v1/matchmaking"
NAMES = {
    f"GET {DSV2}/scopes/{{scope_id}}/entries": "list-scoped-data-store-entries",
    f"POST {DSV2}/scopes/{{scope_id}}/entries": "create-scoped-data-store-entry",
    f"GET {DSV2}/scopes/{{scope_id}}/entries/{{entry_id}}": "get-scoped-data-store-entry",
    f"PATCH {DSV2}/scopes/{{scope_id}}/entries/{{entry_id}}": "update-scoped-data-store-entry",
    f"DELETE {DSV2}/scopes/{{scope_id}}/entries/{{entry_id}}": "delete-scoped-data-store-entry",
    f"POST {DSV2}/scopes/{{scope_id}}/entries/{{entry_id}}:increment": "increment-scoped-data-store-entry",
    f"GET {DSV2}/scopes/{{scope_id}}/entries/{{entry_id}}:listRevisions": "list-scoped-data-store-entry-revisions",
    f"GET {LOGRX}": "find-log-regexes",
    f"POST {LOGRX}": "create-log-regex",
    f"PUT {LOGRX}/{{id}}": "update-log-regex",
    f"DELETE {LOGRX}/{{id}}": "delete-log-regex",
    f"PUT {LOGRX}/{{id}}/ignore": "ignore-log-regex",
    f"PUT {LOGRX}/{{id}}/reorder": "reorder-log-regex",
    "GET /universe-performance-raqi/v1/events/logging/universe/{universeId}/log-details": "get-error-log-details",
    f"GET {DASH}/capabilities": "get-dashboard-capabilities",
    f"GET {DASH}/pinned": "get-pinned-dashboard",
    f"GET {DASH}/{{dashboardId}}": "get-dashboard",
    "GET /notifications/v1/developer-analytics/get-notifs-analytics": "get-notification-analytics",
    "GET /notifications/v1/developer-configuration/experience-notification": "get-experience-notification",
    "GET /notifications/v1/developer-configuration/experience-notifications-list": "list-experience-notifications",
    "POST /notifications/v1/developer-configuration/archive-notification": "archive-notification",
    "POST /notifications/v1/developer-configuration/create-notification": "create-notification",
    "POST /notifications/v1/developer-configuration/update-notification": "update-notification",
    "POST /notifications/v1/experience-notifications/unfollow": "unfollow-experience-notification",
    "GET /cloud/v2/users/{user_id}/operations/{operation_id}": "get-thumbnail-operation",
    "GET /toolbox-service/v2/assets:search": "search-assets-deprecated",
    "POST /toolbox-service/v2/assets:search": "search-assets",
    "GET /toolbox-service/v2/assets/{id}": "get-asset-details",
    f"GET {ALERT}": "list-alerts",
    f"POST {ALERT}": "create-alert",
    f"GET {ALERT}/incidents": "list-alert-incidents",
    f"PATCH {ALERT}/{{alertId}}": "update-alert",
    f"DELETE {ALERT}/{{alertId}}": "delete-alert",
    "POST /analytics-benchmark/v1/universes/{universeId}/benchmarks": "query-benchmarks",
    f"POST {AQ}/dimension-values": "query-dimension-values",
    f"POST {AQ}/metrics": "query-metrics",
    f"GET {AQ}/operations/dimension-values/{{operationId}}": "get-dimension-values-operation",
    f"GET {AQ}/operations/metrics/{{operationId}}": "get-metrics-operation",
    "GET /asset-delivery-api/v1/assetId/{assetId}": "get-asset",
    "GET /asset-delivery-api/v1/assetId/{assetId}/version/{versionNumber}": "get-asset-version",
    "PATCH /asset-permissions-api/v1/assets/permissions": "grant-permissions",
    "GET /cloud/v2/universes/{universeId}/secrets": "list",
    "POST /cloud/v2/universes/{universeId}/secrets": "create",
    "GET /cloud/v2/universes/{universeId}/secrets/public-key": "get-public-key",
    "PATCH /cloud/v2/universes/{universeId}/secrets/{secretId}": "update",
    "DELETE /cloud/v2/universes/{universeId}/secrets/{secretId}": "delete",
    "PATCH /legacy-badges/v1/badges/{badgeId}": "update-badge",
    "POST /legacy-badges/v1/universes/{universeId}/badges": "create-badge",
    "POST /legacy-publish/v1/badges/{badgeId}/icon": "update-badge-icon",
    "GET /legacy-develop/v1/places/{placeId}/teamcreate/active_session/members": "list-active-session-members",
    "GET /legacy-develop/v1/universes/multiget/teamcreate": "list-settings",
    "GET /legacy-develop/v1/universes/{universeId}/teamcreate": "get-settings",
    "PATCH /legacy-develop/v1/universes/{universeId}/teamcreate": "update-settings",
    "DELETE /legacy-develop/v1/universes/{universeId}/teamcreate/memberships": "remove-member",
    "DELETE /legacy-develop/v2/teamtest/{placeId}": "close-team-test",
    "GET /legacy-develop/v1/universes/multiget/permissions": "list-permissions",
    "GET /legacy-develop/v1/universes/{universeId}/permissions": "get-permissions",
    "POST /legacy-develop/v1/universes/{universeId}/activate": "activate",
    "POST /legacy-develop/v1/universes/{universeId}/deactivate": "deactivate",
    "GET /legacy-develop/v1/user/groups/canmanage": "list-manageable",
    "GET /legacy-groups/v1/user/groups/pending": "list-pending-join-requests",
    "POST /legacy-groups/v1/groups/policies": "get-policies",
    "GET /legacy-groups/v1/groups/{groupId}/audit-log": "get-audit-log",
    "PATCH /legacy-groups/v1/groups/{groupId}/description": "update-description",
    "PATCH /legacy-groups/v1/groups/{groupId}/notification-preference": "update-notification-preference",
    "GET /legacy-groups/v1/groups/{groupId}/settings": "get-settings",
    "PATCH /legacy-groups/v1/groups/{groupId}/settings": "update-settings",
    "PATCH /legacy-groups/v1/groups/{groupId}/status": "update-status",
    "GET /legacy-followings/v1/users/{userId}/universes": "list",
    "GET /legacy-followings/v2/users/{userId}/universes": "list-v2",
    "POST /legacy-followings/v1/users/{userId}/universes/{universeId}": "follow",
    "DELETE /legacy-followings/v1/users/{userId}/universes/{universeId}": "unfollow",
    "GET /legacy-followings/v1/users/{userId}/universes/{universeId}/status": "get-status",
    f"GET {GI}/game-icon/games/{{gameId}}": "list-game-icons",
    f"POST {GI}/game-icon/games/{{gameId}}/language-codes/{{languageCode}}": "update-game-icon",
    f"DELETE {GI}/game-icon/games/{{gameId}}/language-codes/{{languageCode}}": "delete-game-icon",
    f"POST {GI}/game-thumbnails/games/{{gameId}}/language-codes/{{languageCode}}/alt-text": "update-game-thumbnail-alt-text",
    f"POST {GI}/game-thumbnails/games/{{gameId}}/language-codes/{{languageCode}}/image": "upload-game-thumbnail",
    f"POST {GI}/game-thumbnails/games/{{gameId}}/language-codes/{{languageCode}}/images/order": "order-game-thumbnails",
    f"DELETE {GI}/game-thumbnails/games/{{gameId}}/language-codes/{{languageCode}}/images/{{imageId}}": "delete-game-thumbnail",
    f"POST {GI}/name-description/games/translation-history": "get-game-name-description-history",
    f"PATCH {GI}/name-description/games/{{gameId}}": "update-game-name-description",
    f"PATCH {GI}/source-language/games/{{gameId}}": "set-source-language",
    f"PATCH {GI}/supported-languages/games/{{gameId}}": "update-supported-languages",
    f"GET {GI}/supported-languages/games/{{gameId}}/automatic-translation-status": "get-automatic-translation-status",
    f"PATCH {GI}/supported-languages/games/{{gameId}}/languages/{{languageCode}}/automatic-translation-status": "set-automatic-translation-status",
    f"PATCH {GI}/supported-languages/games/{{gameId}}/languages/{{languageCode}}/image-translation-status": "set-image-translation-status",
    f"PATCH {GI}/supported-languages/games/{{gameId}}/languages/{{languageCode}}/universe-display-info-automatic-translation-settings": "set-display-info-translation-settings",
    f"GET {GI}/supported-languages/games/{{gameId}}/universe-display-info-automatic-translation-settings": "get-display-info-translation-settings",
    f"POST {LT}/autolocalization/games/{{gameId}}/autolocalizationtable": "create-auto-localization-table",
    f"PATCH {LT}/autolocalization/games/{{gameId}}/settings": "update-auto-localization-settings",
    f"GET {LT}/autolocalization/metadata": "get-auto-localization-metadata",
    f"GET {LT}/localization-table/limits": "get-limits",
    f"GET {LT}/localization-table/tables/{{assetId}}": "get-table-by-asset-id",
    f"GET {LT}/localization-table/tables/{{tableId}}": "get-table",
    f"PATCH {LT}/localization-table/tables/{{tableId}}": "update-table",
    f"GET {LT}/localization-table/tables/{{tableId}}/entries": "list-entries",
    f"POST {LT}/localization-table/tables/{{tableId}}/entries/translation-history": "get-entry-translation-history",
    f"GET {LT}/localization-table/tables/{{tableId}}/entry-count": "get-entry-count",
    "POST /matchmaking-api/v1/game-instances/shutdown": "shutdown-game-instances",
    f"POST {MM}/player-attribute": "create-player-attribute",
    f"DELETE {MM}/player-attribute/{{attributeId}}": "delete-player-attribute",
    f"PATCH {MM}/player-attribute/{{attributeId}}": "update-player-attribute",
    f"GET {MM}/player-attributes/{{universeId}}": "list-player-attributes",
    f"POST {MM}/server-attribute": "create-server-attribute",
    f"DELETE {MM}/server-attribute/{{attributeId}}": "delete-server-attribute",
    f"PATCH {MM}/server-attribute/{{attributeId}}": "update-server-attribute",
    f"GET {MM}/server-attributes/{{universeId}}": "list-server-attributes",
    f"POST {MM}/scoring-configuration": "create-scoring-configuration",
    f"GET {MM}/scoring-configuration/default-weights": "get-default-weights",
    f"GET {MM}/scoring-configuration/generate-mock-servers": "generate-mock-servers",
    f"POST {MM}/scoring-configuration/place": "set-place-scoring-configuration",
    f"DELETE {MM}/scoring-configuration/place/{{placeId}}": "remove-place-scoring-configuration",
    f"DELETE {MM}/scoring-configuration/{{scoringConfigurationId}}": "delete-scoring-configuration",
    f"GET {MM}/scoring-configuration/{{scoringConfigurationId}}": "get-scoring-configuration",
    f"PATCH {MM}/scoring-configuration/{{scoringConfigurationId}}": "update-scoring-configuration",
    f"POST {MM}/scoring-configuration/{{scoringConfigurationId}}/signals": "create-signal",
    f"DELETE {MM}/scoring-configuration/{{scoringConfigurationId}}/signals/{{signalName}}": "delete-signal",
    f"PATCH {MM}/scoring-configuration/{{scoringConfigurationId}}/signals/{{signalName}}": "update-signal",
    f"GET {MM}/scoring-configurations/{{universeId}}": "list-scoring-configurations",
    f"GET {MM}/scoring-configurations/{{universeId}}/places": "list-scoring-configuration-places",
    f"GET {MM}/universe/{{universeId}}/feature-flags": "get-feature-flags",
    "GET /v1/asset-types/{assetType}/agents": "list-asset-type-agents",
    "GET /v1/assets-thumbnail-3d": "get-asset-3d",
    "GET /v1/users/avatar-3d": "get-avatar-3d",
    "GET /v1/users/outfit-3d": "get-outfit-3d",
    "POST /v1/avatar-item-updates/create-context": "create-item-update-context",
    "POST /v1/bundles": "create",
    "POST /v1/bundles/create-context": "create-context",
    "GET /v1/bundles/metadata": "get-metadata",
    "GET /v1/bundles/status/{creationStatusId}": "get-creation-status",
    "POST /v1/collectibles": "publish",
    "PATCH /v1/collectibles/bulk-update": "bulk-update",
    "GET /v1/collectibles/check-item-configuration-access": "check-item-configuration-access",
    "GET /v1/collectibles/check-item-publish-access": "check-item-publish-access",
    "GET /v1/collectibles/check-item-upload-access": "check-item-upload-access",
    "GET /v1/collectibles/commission-rates": "get-commission-rates",
    "GET /v1/collectibles/metadata": "get-metadata",
    "GET /v1/collectibles/price-floor": "get-price-floor",
    "GET /v1/collectibles/publishing-fees": "get-publishing-fees",
    "GET /v1/collectibles/publishing-fees/preview": "preview-publishing-fees",
    "GET /v1/collectibles/regional-pricing/preview": "preview-regional-pricing",
    "GET /v1/collectibles/regional-pricing/representative-countries": "list-representative-countries",
    "GET /v1/collectibles/rental-pricing/preview": "preview-rental-pricing",
    "GET /v1/collectibles/revenue-split": "get-revenue-split",
    "PATCH /v1/collectibles/update-item-display-information": "update-item-display-information",
    "PATCH /v1/collectibles/{collectibleItemId}": "update",
    "POST /v1/collectibles/{collectibleItemId}/restock": "restock",
    "GET /v1/collectibles/{collectibleItemId}/restock-eligibility": "get-restock-eligibility",
    "GET /v1/collectibles/{collectibleItemId}/restocking-fees": "get-restocking-fees",
    "DELETE /v1/collectibles/{collectibleItemId}/sale-schedule": "cancel-sale-schedule",
    "POST /v1/creations/get-asset-details": "get-asset-details",
    "GET /v1/creations/get-assets": "list-created-assets",
    "POST /v1/delist-item": "delist-item",
    "GET /v1/folders": "list",
    "POST /v1/folders": "create",
    "DELETE /v1/folders/{folderId}": "delete",
    "GET /v1/folders/{folderId}": "get",
    "PATCH /v1/folders/{folderId}": "update",
    "DELETE /v1/folders/{folderId}/items": "remove-items",
    "GET /v1/folders/{folderId}/items": "list-items",
    "POST /v1/folders/{folderId}/items": "add-items",
    "GET /v1/items": "get",
    "GET /v1/items/by-creator": "list-by-creator",
    "GET /v1/items/by-token": "list-by-token",
    "GET /v1/items/categories": "list-categories",
    "GET /v1/items/price-floor": "get-price-floor",
    "GET /v1/items/upload-fee": "get-upload-fee",
    "GET /v1/permissions/action-allowed-for-item-type": "check-action-allowed",
    "GET /v1/permissions/groups": "list-allowed-groups",
    "GET /v1/permissions/item-types": "list-allowed-item-types",
}

# Localized name/description/icon endpoints repeat for three resource kinds.
for _seg, _noun in (("badges/{badgeId}", "badge"),
                    ("developer-products/{developerProductId}", "developer-product"),
                    ("game-passes/{gamePassId}", "game-pass")):
    _b = f"{GI}/{_seg}"
    _lc = "language-codes/{languageCode}"
    NAMES.update({
        f"PATCH {_b}/description/{_lc}": f"update-{_noun}-description",
        f"GET {_b}/icons": f"list-{_noun}-icons",
        f"DELETE {_b}/icons/{_lc}": f"delete-{_noun}-icon",
        f"POST {_b}/icons/{_lc}": f"update-{_noun}-icon",
        f"GET {_b}/name-description": f"list-{_noun}-name-descriptions",
        f"DELETE {_b}/name-description/{_lc}": f"delete-{_noun}-name-description",
        f"PATCH {_b}/name-description/{_lc}": f"update-{_noun}-name-description",
        f"PATCH {_b}/name/{_lc}": f"update-{_noun}-name",
    })


def kebab(s):
    s = re.sub(r"([a-z0-9])([A-Z])", r"\1-\2", s)
    s = re.sub(r"([A-Z]+)([A-Z][a-z])", r"\1-\2", s)
    return re.sub(r"[^a-z0-9]+", "-", s.lower()).strip("-")


def auth_of(security):
    """Normalise an OpenAPI security requirement list to scheme names."""
    if security is None:
        return ["unspecified"]
    out = set()
    for req in security:
        if not req:
            out.add("none")
        for k in req:
            out.add(SCHEME[k])
    return [a for a in AUTH_ORDER if a in out]


def api_key_eligible(security):
    # Plan rule: a requirement object that is exactly {"roblox-api-key": []}.
    return any(req == {"roblox-api-key": []} for req in (security or []))


def family_of(path):
    for rx, fam, group in FAMILIES:
        if re.search(rx, path):
            return fam, group
    raise SystemExit(f"no family rule for {path}")


def command_of(op_id, spec_op_id):
    if op_id in NAMES:
        return NAMES[op_id]
    if not spec_op_id:
        raise SystemExit(f"no planned command name for {op_id}")
    base = re.split(r"[_.]", spec_op_id.split("__Using_")[0])[-1]
    name = kebab(base)
    return re.sub(r"-async$", "", name)


def seed_owner(op_id, path, op):
    family, group = family_of(path)
    owner = {"family": family}
    if op_id in EXISTING:
        cli, aliases = EXISTING[op_id]
        owner["cli"] = f"rbxcloud {cli}"
        if aliases:
            owner["cli_aliases"] = [f"rbxcloud {a}" for a in aliases]
        owner["status"] = "existing"
    elif op_id in DEFECTIVE:
        owner["cli"] = f"rbxcloud {DEFECTIVE[op_id]}"
        owner["status"] = "defective"
    else:
        owner["cli"] = f"rbxcloud {group} {command_of(op_id, op.get('operationId'))}"
        owner["status"] = "pending"
    if op_id in NOTES:
        owner["note"] = NOTES[op_id]
    return owner


def build(spec_raw, previous):
    """Return the manifest for this spec, keeping owner fields from `previous`."""
    spec = json.loads(spec_raw)
    kept = {r["id"]: r for r in (previous or {}).get("operations", [])}

    operations, excluded = [], []
    for path in sorted(spec["paths"]):
        item = spec["paths"][path]
        for method in METHODS:
            if method not in item:
                continue
            op = item[method]
            m = method.upper()
            op_id = f"{m} {path}"
            auth = auth_of(op.get("security"))
            if not api_key_eligible(op.get("security")):
                assert "api-key" not in auth, op_id
                reason = ("legacy-cookie" if "legacy-cookie" in auth
                          else "oauth2-only" if "oauth2" in auth
                          else "no-api-key")
                excluded.append({"id": op_id, "method": m, "path": path,
                                 "auth": auth, "reason": reason})
                continue
            if op_id in kept:
                owner = {k: kept[op_id][k] for k in OWNER_FIELDS if k in kept[op_id]}
            else:
                owner = seed_owner(op_id, path, op)
            servers = op.get("servers") or item.get("servers") or spec["servers"]
            assert len(servers) == 1, op_id
            rec = {"id": op_id, "method": m, "path": path, "auth": auth,
                   "family": owner["family"], "cli": owner["cli"]}
            if "cli_aliases" in owner:
                rec["cli_aliases"] = owner["cli_aliases"]
            rec["status"] = owner["status"]
            rec["operation_id"] = op.get("operationId")
            rec["tags"] = op.get("tags", [])
            rec["server"] = servers[0]["url"]
            rec["stability"] = op.get("x-roblox-stability")
            if "note" in owner:
                rec["note"] = owner["note"]
            operations.append(rec)

    ids = {r["id"] for r in operations}
    if previous is None:
        # Seed tables must describe this spec exactly when building from scratch.
        for table in (EXISTING, DEFECTIVE, NAMES, NOTES):
            missing = sorted(set(table) - ids)
            if missing:
                raise SystemExit("table ids not API-key operations in spec:\n  " + "\n  ".join(missing))
    gone = sorted(set(kept) - ids)
    for op_id in gone:
        r = kept[op_id]
        print(f"left the API-key set: {op_id} (was {r['status']}, {r['cli']})", file=sys.stderr)

    outside = (previous or {}).get("outside_spec", OUTSIDE_SPEC)
    validate(operations, outside)

    counts = collections.Counter(r["status"] for r in operations)
    excl_reasons = collections.Counter(r["reason"] for r in excluded)
    summary = {
        "spec_operations": len(operations) + len(excluded),
        "api_key_operations": len(operations),
        "covered": sum(counts[s] for s in STATUSES if s in COVERED),
        "not_covered": sum(counts[s] for s in STATUSES if s not in COVERED),
    }
    for s in STATUSES:
        summary[s] = counts[s]
    summary["excluded"] = len(excluded)
    summary["excluded_by_reason"] = dict(sorted(excl_reasons.items()))
    return {
        "schema_version": 1,
        "spec": {"path": SPEC_PATH, "source_url": SOURCE_URL,
                 "openapi": spec["openapi"], "default_server": DEFAULT_SERVER},
        "spec_sha256": hashlib.sha256(spec_raw).hexdigest(),
        "summary": summary,
        "outside_spec": outside,
        "operations": operations,
        "excluded": excluded,
    }


def validate(operations, outside):
    errors = []
    planned = collections.Counter()
    real = set(o["cli"] for o in outside)
    for r in operations:
        if r["status"] not in STATUSES:
            errors.append(f"{r['id']}: unknown status {r['status']!r}")
        if r["status"] == "defective" and not r.get("note"):
            errors.append(f"{r['id']}: defective records need a note")
        if r.get("cli_aliases") and r["status"] == "pending":
            errors.append(f"{r['id']}: pending records cannot have cli_aliases")
        if r["status"] == "pending":
            planned[r["cli"]] += 1
        else:
            real.add(r["cli"])
            real.update(r.get("cli_aliases", []))
    errors += [f"planned command owns several operations: {c}" for c, n in sorted(planned.items()) if n > 1]
    errors += [f"planned command is already a real command: {c}" for c in sorted(set(planned) & real)]
    if errors:
        raise SystemExit("\n".join(errors))


def render_manifest(m):
    out = ["{\n"]
    for k in ("schema_version", "spec", "spec_sha256", "summary"):
        body = json.dumps(m[k], indent=2).replace("\n", "\n  ")
        out.append(f"  {json.dumps(k)}: {body},\n")
    out.append('  "outside_spec": [\n')
    out.append(",\n".join("    " + json.dumps(r) for r in m["outside_spec"]))
    out.append('\n  ],\n  "operations": [\n')
    out.append(",\n".join("    " + json.dumps(r, indent=2).replace("\n", "\n    ")
                          for r in m["operations"]))
    out.append('\n  ],\n  "excluded": [\n')
    out.append(",\n".join("    " + json.dumps(r) for r in m["excluded"]))
    out.append("\n  ]\n}\n")
    return "".join(out)


def render_doc(m):
    """docs/COVERAGE.md, from the manifest alone."""
    s = m["summary"]
    ops = m["operations"]
    fam = collections.OrderedDict()
    for r in sorted(ops, key=lambda r: r["family"]):
        f = fam.setdefault(r["family"], {"counts": collections.Counter(), "groups": set()})
        f["counts"][r["status"]] += 1
        f["groups"].add(r["cli"].split()[1])
    servers = collections.Counter(r["server"] for r in ops)
    L = []
    w = L.append
    w("# Open Cloud coverage")
    w("")
    w("Generated from `openapi/coverage.json` by `python3 scripts/gen_coverage.py`. "
      "Do not edit by hand: edit the manifest, then run the script.")
    w("")
    w("## Baseline")
    w("")
    w(f"- Spec snapshot: `{m['spec']['path']}` (OpenAPI {m['spec']['openapi']})")
    w(f"- Upstream: <{m['spec']['source_url']}>")
    w(f"- SHA-256: `{m['spec_sha256']}`")
    w("")
    w("## Totals")
    w("")
    w("| Set | Operations |")
    w("|---|---:|")
    w(f"| Operations in the spec | {s['spec_operations']} |")
    w(f"| Accept an API key (coverage target) | {s['api_key_operations']} |")
    w(f"| Covered (`existing` + `implemented`) | {s['covered']} |")
    w(f"| Not covered (`defective` + `pending`) | {s['not_covered']} |")
    w(f"| Excluded (no API-key auth) | {s['excluded']} |")
    w("")
    w("An operation is in the target when its `security` list contains exactly "
      "`{\"roblox-api-key\": []}` as one alternative. Operations that also accept "
      "OAuth 2.0 or a legacy cookie stay in the target; operations that never "
      "accept an API key are listed under `excluded` in the manifest and are never "
      "counted as required.")
    w("")
    w("| Exclusion reason | Operations |")
    w("|---|---:|")
    for k, v in s["excluded_by_reason"].items():
        w(f"| `{k}` | {v} |")
    w("")
    w("## Statuses")
    w("")
    w("| Status | Operations | Covered | Meaning |")
    w("|---|---:|---|---|")
    w(f"| `existing` | {s['existing']} | yes | A legacy client (`src/rbx/v1`, `src/rbx/v2`) and its command reach this method and path. Request parameters may be incomplete; see Notes. No new work may use this status. |")
    w(f"| `implemented` | {s['implemented']} | yes | A typed method on the shared transport plus an explicit command. Every newly built or migrated operation gets this status. |")
    w(f"| `defective` | {s['defective']} | no | A legacy command targets this operation but does not send the documented request. Needs a `note`. |")
    w(f"| `pending` | {s['pending']} | no | No owner code yet. `cli` is the planned command and may still be renamed. |")
    w("")
    w("## Families")
    w("")
    w("`family` is the typed client module that owns the operation. `cli` is the "
      "explicit command that owns it: the real command for `existing`, `implemented` "
      "and `defective` records, the planned command for `pending` ones. No record is "
      "owned by a generic request command.")
    w("")
    w("| Family | CLI group | Existing | Implemented | Defective | Pending | Total |")
    w("|---|---|---:|---:|---:|---:|---:|")
    for name, f in fam.items():
        groups = ", ".join(f"`{g}`" for g in sorted(f["groups"]))
        c = f["counts"]
        cells = " | ".join(str(c[x]) for x in STATUSES)
        w(f"| `{name}` | {groups} | {cells} | {sum(c.values())} |")
    cells = " | ".join(f"**{s[x]}**" for x in STATUSES)
    w(f"| **Total** | | {cells} | **{s['api_key_operations']}** |")
    w("")
    w("## Servers")
    w("")
    w("`id` is `HTTP_METHOD /path` and does not include the host. An endpoint is "
      "located by `server` plus `path`. Operations per server:")
    w("")
    w("| Server | Operations |")
    w("|---|---:|")
    for url, n in sorted(servers.items(), key=lambda kv: (-kv[1], kv[0])):
        w(f"| `{url}` | {n} |")
    w("")
    w("## Legacy commands outside the spec")
    w("")
    w("These commands send requests that have no API-key operation in this spec "
      "snapshot. They are kept for compatibility and are not counted above.")
    w("")
    w("| Command | When | Request |")
    w("|---|---|---|")
    for o in m["outside_spec"]:
        w(f"| `{o['cli']}` | {o['when'] or 'always'} | `{o['request']}` |")
    w("")
    w("## Notes")
    w("")
    w("Known mismatches between an owner and the spec. Legacy clients are not "
      "changed here; each mismatch is fixed when its family is migrated.")
    w("")
    w("| Operation | Status | Command | Note |")
    w("|---|---|---|---|")
    for r in ops:
        if "note" in r:
            w(f"| `{r['id']}` | `{r['status']}` | `{r['cli']}` | {r['note']} |")
    w("")
    w("## Manifest schema")
    w("")
    w("`openapi/coverage.json` has `schema_version`, `spec`, `spec_sha256`, "
      "`summary`, `outside_spec`, `operations` and `excluded`.")
    w("")
    w("| Field | Meaning |")
    w("|---|---|")
    w("| `id` | Stable key, `HTTP_METHOD /path` exactly as in the spec |")
    w("| `method`, `path` | The two halves of `id` |")
    w("| `auth` | Accepted schemes, see below |")
    w("| `family` | Typed client module that owns the operation |")
    w("| `cli` | Owning command, `rbxcloud <group> <command>` |")
    w("| `cli_aliases` | Other real commands that call the same operation |")
    w("| `status` | `existing`, `implemented`, `defective` or `pending` |")
    w("| `operation_id`, `tags`, `server`, `stability` | Copied from the spec |")
    w("| `note` | Known mismatch between the owner and the spec |")
    w("")
    w("`excluded` records carry `id`, `method`, `path`, `auth` and `reason` only.")
    w("")
    w("| `auth` value | Spec `security` |")
    w("|---|---|")
    w("| `api-key` | `roblox-api-key`, the `x-api-key` header |")
    w("| `oauth2` | `roblox-oauth2` |")
    w("| `legacy-cookie` | `roblox-legacy-cookie`, the `.ROBLOSECURITY` cookie |")
    w("| `none` | An empty requirement `{}`: callable without credentials |")
    w("| `unspecified` | The operation has no `security` key. Only on `excluded` records |")
    w("")
    w("## Maintenance")
    w("")
    w("- Change coverage: edit the record in `openapi/coverage.json` (`status`, "
      "`cli`, `cli_aliases`, `family`, `note`), then run "
      "`python3 scripts/gen_coverage.py` to refresh `summary` and this page.")
    w("- Refresh the spec: replace `openapi/roblox.json`, then run the script. "
      "Owner fields of operations already in the manifest are kept; new operations "
      "are seeded as `pending`.")
    w("- `cargo test --test coverage_manifest` checks the manifest against the spec "
      "snapshot and this page against the manifest.")
    return "\n".join(L) + "\n"


def read(path, mode="r"):
    kw = {} if "b" in mode else {"encoding": "utf-8", "newline": ""}
    with open(path, mode, **kw) as f:
        return f.read()


def main():
    args = set(sys.argv[1:])
    unknown = args - {"--check", "--reseed", "--list"}
    if unknown:
        raise SystemExit(f"unknown arguments: {sorted(unknown)}")
    previous = None
    if "--reseed" not in args and os.path.exists(MANIFEST_PATH):
        previous = json.loads(read(MANIFEST_PATH))
    manifest = build(read(SPEC_PATH, "rb"), previous)

    if "--list" in args:
        for r in manifest["operations"]:
            print(f"{r['status']:11} {r['family']:26} {r['cli']:70} {r['id']}")
        return

    outputs = {MANIFEST_PATH: render_manifest(manifest), DOC_PATH: render_doc(manifest)}
    if "--check" in args:
        stale = [p for p, text in outputs.items()
                 if not os.path.exists(p) or read(p) != text]
        if stale:
            raise SystemExit("out of date, run python3 scripts/gen_coverage.py: " + ", ".join(stale))
        return
    for path, text in outputs.items():
        with open(path, "w", encoding="utf-8", newline="\n") as f:
            f.write(text)
    print(json.dumps(manifest["summary"], indent=2))


if __name__ == "__main__":
    main()
