use clap::{Args, Subcommand};
use reqwest::Method;

struct Operation {
    name: &'static str,
    method: &'static str,
    path: &'static str,
    summary: &'static str,
    params: &'static [&'static str],
}

const OPERATIONS: &[Operation] = &[
    Operation { name: "accept-group-join-request", method: "POST", path: "/cloud/v2/groups/{group_id}/join-requests/{join_request_id}:accept", summary: "Accept Group Join Request", params: &["group_id", "join_request_id"] },
    Operation { name: "activates-a-universes", method: "POST", path: "/legacy-develop/v1/universes/{universeId}/activate", summary: "Activates a universes.", params: &["universeId"] },
    Operation { name: "add-or-remove-supported-languages-for-a-game", method: "PATCH", path: "/legacy-game-internationalization/v1/supported-languages/games/{gameId}", summary: "Add or remove supported languages for a game.", params: &["gameId"] },
    Operation { name: "archives-the-asset", method: "POST", path: "/assets/v1/assets/{assetId}:archive", summary: "Archives the asset.", params: &["assetId"] },
    Operation { name: "assign-role-group-membership", method: "POST", path: "/cloud/v2/groups/{group_id}/memberships/{membership_id}:assignRole", summary: "Assign Role Group Membership", params: &["group_id", "membership_id"] },
    Operation { name: "bulk-deletes-saves-max-of-5000-saves-per-request", method: "POST", path: "/toolbox-service/v1/saves:bulkDelete", summary: "Bulk deletes saves.
Max of 5000 saves per request.", params: &[] },
    Operation { name: "close-a-game-instance-that-is-being-used-for-team-testing", method: "DELETE", path: "/legacy-develop/v2/teamtest/{placeId}", summary: "Close a game instance that is being used for team testing", params: &["placeId"] },
    Operation { name: "create-creator-store-product", method: "POST", path: "/cloud/v2/creator-store-products", summary: "Create Creator Store Product", params: &[] },
    Operation { name: "create-data-store-entry", method: "POST", path: "/cloud/v2/universes/{universe_id}/data-stores/{data_store_id}/entries", summary: "Create Data Store Entry", params: &["universe_id", "data_store_id"] },
    Operation { name: "create-data-store-entry-scopes-by-scope-id-entries", method: "POST", path: "/cloud/v2/universes/{universe_id}/data-stores/{data_store_id}/scopes/{scope_id}/entries", summary: "Create Data Store Entry", params: &["universe_id", "data_store_id", "scope_id"] },
    Operation { name: "create-developer-product", method: "POST", path: "/developer-products/v2/universes/{universeId}/developer-products", summary: "Create developer product", params: &["universeId"] },
    Operation { name: "create-game-pass", method: "POST", path: "/game-passes/v1/universes/{universeId}/game-passes", summary: "Create game pass", params: &["universeId"] },
    Operation { name: "create-luau-execution-session-task", method: "POST", path: "/cloud/v2/universes/{universe_id}/places/{place_id}/luau-execution-session-tasks", summary: "Create Luau Execution Session Task", params: &["universe_id", "place_id"] },
    Operation { name: "create-luau-execution-session-task-binary-input", method: "POST", path: "/cloud/v2/universes/{universe_id}/luau-execution-session-task-binary-inputs", summary: "Create Luau Execution Session Task Binary Input", params: &["universe_id"] },
    Operation { name: "create-luau-execution-session-task-versions-by-version-id-luau-execution-session-tasks", method: "POST", path: "/cloud/v2/universes/{universe_id}/places/{place_id}/versions/{version_id}/luau-execution-session-tasks", summary: "Create Luau Execution Session Task", params: &["universe_id", "place_id", "version_id"] },
    Operation { name: "create-memory-store-queue-item", method: "POST", path: "/cloud/v2/universes/{universe_id}/memory-store/queues/{queue_id}/items", summary: "Create Memory Store Queue Item", params: &["universe_id", "queue_id"] },
    Operation { name: "create-memory-store-sorted-map-item", method: "POST", path: "/cloud/v2/universes/{universe_id}/memory-store/sorted-maps/{sorted_map_id}/items", summary: "Create Memory Store Sorted Map Item", params: &["universe_id", "sorted_map_id"] },
    Operation { name: "create-ordered-data-store-entry", method: "POST", path: "/cloud/v2/universes/{universe_id}/ordered-data-stores/{ordered_data_store_id}/scopes/{scope_id}/entries", summary: "Create Ordered Data Store Entry", params: &["universe_id", "ordered_data_store_id", "scope_id"] },
    Operation { name: "create-secret", method: "POST", path: "/cloud/v2/universes/{universeId}/secrets", summary: "Create Secret", params: &["universeId"] },
    Operation { name: "create-user-notification", method: "POST", path: "/cloud/v2/users/{user_id}/notifications", summary: "Create User Notification", params: &["user_id"] },
    Operation { name: "creates-a-new-badge", method: "POST", path: "/legacy-badges/v1/universes/{universeId}/badges", summary: "Creates a new badge.", params: &["universeId"] },
    Operation { name: "creates-a-save", method: "POST", path: "/toolbox-service/v1/saves", summary: "Creates a save.", params: &[] },
    Operation { name: "creates-an-asset-with-provided-content-and-metadata", method: "POST", path: "/assets/v1/assets", summary: "Creates an asset with provided content and metadata.", params: &[] },
    Operation { name: "creates-the-following-between-a-user-with-userid-and-universe-with-universeid", method: "POST", path: "/legacy-followings/v1/users/{userId}/universes/{universeId}", summary: "Creates the following between a user with userId and universe with universeId", params: &["userId", "universeId"] },
    Operation { name: "deactivates-a-universe", method: "POST", path: "/legacy-develop/v1/universes/{universeId}/deactivate", summary: "Deactivates a universe.", params: &["universeId"] },
    Operation { name: "decline-group-join-request", method: "POST", path: "/cloud/v2/groups/{group_id}/join-requests/{join_request_id}:decline", summary: "Decline Group Join Request", params: &["group_id", "join_request_id"] },
    Operation { name: "delete-a-localized-icon-from-a-badge", method: "DELETE", path: "/legacy-game-internationalization/v1/badges/{badgeId}/icons/language-codes/{languageCode}", summary: "Delete a localized icon from a badge", params: &["badgeId", "languageCode"] },
    Operation { name: "delete-a-localized-icon-from-a-developer-product", method: "DELETE", path: "/legacy-game-internationalization/v1/developer-products/{developerProductId}/icons/language-codes/{languageCode}", summary: "Delete a localized icon from a developer product", params: &["developerProductId", "languageCode"] },
    Operation { name: "delete-a-localized-icon-from-a-game", method: "DELETE", path: "/legacy-game-internationalization/v1/game-icon/games/{gameId}/language-codes/{languageCode}", summary: "Delete a localized icon from a game", params: &["gameId", "languageCode"] },
    Operation { name: "delete-a-localized-icon-from-a-game-pass", method: "DELETE", path: "/legacy-game-internationalization/v1/game-passes/{gamePassId}/icons/language-codes/{languageCode}", summary: "Delete a localized icon from a game pass", params: &["gamePassId", "languageCode"] },
    Operation { name: "delete-data-store", method: "DELETE", path: "/cloud/v2/universes/{universe_id}/data-stores/{data_store_id}", summary: "Delete Data Store", params: &["universe_id", "data_store_id"] },
    Operation { name: "delete-data-store-entry", method: "DELETE", path: "/cloud/v2/universes/{universe_id}/data-stores/{data_store_id}/entries/{entry_id}", summary: "Delete Data Store Entry", params: &["universe_id", "data_store_id", "entry_id"] },
    Operation { name: "delete-data-store-entry-by-scope-id-entries-by-entry-id", method: "DELETE", path: "/cloud/v2/universes/{universe_id}/data-stores/{data_store_id}/scopes/{scope_id}/entries/{entry_id}", summary: "Delete Data Store Entry", params: &["universe_id", "data_store_id", "scope_id", "entry_id"] },
    Operation { name: "delete-entry", method: "DELETE", path: "/datastores/v1/universes/{universeId}/standard-datastores/datastore/entries/entry", summary: "Delete entry.", params: &["universeId"] },
    Operation { name: "delete-localized-name-and-description-of-a-badge", method: "DELETE", path: "/legacy-game-internationalization/v1/badges/{badgeId}/name-description/language-codes/{languageCode}", summary: "Delete localized name and description of a badge", params: &["badgeId", "languageCode"] },
    Operation { name: "delete-localized-name-and-description-of-a-developer-product", method: "DELETE", path: "/legacy-game-internationalization/v1/developer-products/{developerProductId}/name-description/language-codes/{languageCode}", summary: "Delete localized name and description of a developer product", params: &["developerProductId", "languageCode"] },
    Operation { name: "delete-localized-name-and-description-of-a-game-pass", method: "DELETE", path: "/legacy-game-internationalization/v1/game-passes/{gamePassId}/name-description/language-codes/{languageCode}", summary: "Delete localized name and description of a game pass", params: &["gamePassId", "languageCode"] },
    Operation { name: "delete-memory-store-sorted-map-item", method: "DELETE", path: "/cloud/v2/universes/{universe_id}/memory-store/sorted-maps/{sorted_map_id}/items/{item_id}", summary: "Delete Memory Store Sorted Map Item", params: &["universe_id", "sorted_map_id", "item_id"] },
    Operation { name: "delete-ordered-data-store-entry", method: "DELETE", path: "/cloud/v2/universes/{universe_id}/ordered-data-stores/{ordered_data_store_id}/scopes/{scope_id}/entries/{entry_id}", summary: "Delete Ordered Data Store Entry", params: &["universe_id", "ordered_data_store_id", "scope_id", "entry_id"] },
    Operation { name: "delete-secret", method: "DELETE", path: "/cloud/v2/universes/{universeId}/secrets/{secretId}", summary: "Delete Secret", params: &["universeId", "secretId"] },
    Operation { name: "deletes-a-save", method: "DELETE", path: "/toolbox-service/v1/saves", summary: "Deletes a save.", params: &[] },
    Operation { name: "deletes-the-following-between-a-user-with-userid-and-universe-with-universeid", method: "DELETE", path: "/legacy-followings/v1/users/{userId}/universes/{universeId}", summary: "Deletes the following between a user with userId and universe with universeId", params: &["userId", "universeId"] },
    Operation { name: "deletes-the-game-thumbnail", method: "DELETE", path: "/legacy-game-internationalization/v1/game-thumbnails/games/{gameId}/language-codes/{languageCode}/images/{imageId}", summary: "Deletes the game thumbnail.", params: &["gameId", "languageCode", "imageId"] },
    Operation { name: "discard-memory-store-queue-items", method: "POST", path: "/cloud/v2/universes/{universe_id}/memory-store/queues/{queue_id}/items:discard", summary: "Discard Memory Store Queue Items", params: &["universe_id", "queue_id"] },
    Operation { name: "edit-team-create-settings-for-a-universe", method: "PATCH", path: "/legacy-develop/v1/universes/{universeId}/teamcreate", summary: "Edit team create settings for a universe.", params: &["universeId"] },
    Operation { name: "enable-or-disable-automatic-translation-for-a-game-and-language", method: "PATCH", path: "/legacy-game-internationalization/v1/supported-languages/games/{gameId}/languages/{languageCode}/automatic-translation-status", summary: "Enable or disable automatic translation for a game and language.", params: &["gameId", "languageCode"] },
    Operation { name: "enable-or-disable-image-translation-for-a-game-and-language", method: "PATCH", path: "/legacy-game-internationalization/v1/supported-languages/games/{gameId}/languages/{languageCode}/image-translation-status", summary: "Enable or disable image translation for a game and language.", params: &["gameId", "languageCode"] },
    Operation { name: "flush-memory-store", method: "POST", path: "/cloud/v2/universes/{universe_id}/memory-store:flush", summary: "Flush Memory Store", params: &["universe_id"] },
    Operation { name: "forecast-the-impact-of-restarting-game-servers-for-a-universe", method: "GET", path: "/server-management/v1/universes/{universeId}/restarts:forecast", summary: "Forecast the impact of restarting game servers for a universe.", params: &["universeId"] },
    Operation { name: "generate-speech-asset", method: "POST", path: "/cloud/v2/universes/{universe_id}:generateSpeechAsset", summary: "Generate Speech Asset", params: &["universe_id"] },
    Operation { name: "generate-user-thumbnail", method: "GET", path: "/cloud/v2/users/{user_id}:generateThumbnail", summary: "Generate User Thumbnail", params: &["user_id"] },
    Operation { name: "get-3d-object-for-an-outfit", method: "GET", path: "/v1/users/outfit-3d", summary: "Get 3d object for an outfit", params: &[] },
    Operation { name: "get-all-icons-for-a-badge", method: "GET", path: "/legacy-game-internationalization/v1/badges/{badgeId}/icons", summary: "Get all icons for a badge", params: &["badgeId"] },
    Operation { name: "get-all-icons-for-a-developer-product", method: "GET", path: "/legacy-game-internationalization/v1/developer-products/{developerProductId}/icons", summary: "Get all icons for a developer product", params: &["developerProductId"] },
    Operation { name: "get-all-icons-for-a-game", method: "GET", path: "/legacy-game-internationalization/v1/game-icon/games/{gameId}", summary: "Get all icons for a game", params: &["gameId"] },
    Operation { name: "get-all-icons-for-a-game-pass", method: "GET", path: "/legacy-game-internationalization/v1/game-passes/{gamePassId}/icons", summary: "Get all icons for a game pass", params: &["gamePassId"] },
    Operation { name: "get-all-names-and-descriptions-of-a-developer-product", method: "GET", path: "/legacy-game-internationalization/v1/developer-products/{developerProductId}/name-description", summary: "Get all names and descriptions of a developer product", params: &["developerProductId"] },
    Operation { name: "get-all-names-and-descriptions-of-a-game-pass", method: "GET", path: "/legacy-game-internationalization/v1/game-passes/{gamePassId}/name-description", summary: "Get all names and descriptions of a game pass", params: &["gamePassId"] },
    Operation { name: "get-asset-version", method: "GET", path: "/assets/v1/assets/{assetId}/versions/{versionNumber}", summary: "Get Asset Version", params: &["assetId", "versionNumber"] },
    Operation { name: "get-avatar-3d-object-for-a-user", method: "GET", path: "/v1/users/avatar-3d", summary: "Get Avatar 3d object for a user", params: &[] },
    Operation { name: "get-creator-store-asset-details", method: "GET", path: "/toolbox-service/v2/assets/{id}", summary: "Get Creator Store Asset Details", params: &["id"] },
    Operation { name: "get-creator-store-product", method: "GET", path: "/cloud/v2/creator-store-products/{creator_store_product_id}", summary: "Get Creator Store Product", params: &["creator_store_product_id"] },
    Operation { name: "get-data-store-entry", method: "GET", path: "/cloud/v2/universes/{universe_id}/data-stores/{data_store_id}/entries/{entry_id}", summary: "Get Data Store Entry", params: &["universe_id", "data_store_id", "entry_id"] },
    Operation { name: "get-data-store-entry-by-scope-id-entries-by-entry-id", method: "GET", path: "/cloud/v2/universes/{universe_id}/data-stores/{data_store_id}/scopes/{scope_id}/entries/{entry_id}", summary: "Get Data Store Entry", params: &["universe_id", "data_store_id", "scope_id", "entry_id"] },
    Operation { name: "get-developer-product-with-configuration-details", method: "GET", path: "/developer-products/v2/universes/{universeId}/developer-products/{productId}/creator", summary: "Get developer product with configuration details", params: &["universeId", "productId"] },
    Operation { name: "get-entry", method: "GET", path: "/datastores/v1/universes/{universeId}/standard-datastores/datastore/entries/entry", summary: "Get entry.", params: &["universeId"] },
    Operation { name: "get-entry-version", method: "GET", path: "/datastores/v1/universes/{universeId}/standard-datastores/datastore/entries/entry/versions/version", summary: "Get entry version.", params: &["universeId"] },
    Operation { name: "get-game-pass-with-configuration-details", method: "GET", path: "/game-passes/v1/universes/{universeId}/game-passes/{gamePassId}/creator", summary: "Get game pass with configuration details", params: &["universeId", "gamePassId"] },
    Operation { name: "get-group", method: "GET", path: "/cloud/v2/groups/{group_id}", summary: "Get Group", params: &["group_id"] },
    Operation { name: "get-group-role", method: "GET", path: "/cloud/v2/groups/{group_id}/roles/{role_id}", summary: "Get Group Role", params: &["group_id", "role_id"] },
    Operation { name: "get-instance", method: "GET", path: "/cloud/v2/universes/{universe_id}/places/{place_id}/instances/{instance_id}", summary: "Get Instance", params: &["universe_id", "place_id", "instance_id"] },
    Operation { name: "get-legacy-game-internationalization-v1-badges-badgeid-name-description", method: "GET", path: "/legacy-game-internationalization/v1/badges/{badgeId}/name-description", summary: "get /legacy-game-internationalization/v1/badges/{badgeId}/name-description", params: &["badgeId"] },
    Operation { name: "get-limits-for-translation-table-entries-operations", method: "GET", path: "/legacy-localization-tables/v1/localization-table/limits", summary: "Get limits for translation table entries operations", params: &[] },
    Operation { name: "get-luau-execution-session-task", method: "GET", path: "/cloud/v2/universes/{universe_id}/places/{place_id}/versions/{version_id}/luau-execution-sessions/{luau_execution_session_id}/tasks/{task_id}", summary: "Get Luau Execution Session Task", params: &["universe_id", "place_id", "version_id", "luau_execution_session_id", "task_id"] },
    Operation { name: "get-memory-store-flush-operation", method: "GET", path: "/cloud/v2/universes/{universe_id}/memory-store/operations/{operation_id}", summary: "Get Memory Store Flush Operation", params: &["universe_id", "operation_id"] },
    Operation { name: "get-memory-store-sorted-map-item", method: "GET", path: "/cloud/v2/universes/{universe_id}/memory-store/sorted-maps/{sorted_map_id}/items/{item_id}", summary: "Get Memory Store Sorted Map Item", params: &["universe_id", "sorted_map_id", "item_id"] },
    Operation { name: "get-ordered-data-store-entry", method: "GET", path: "/cloud/v2/universes/{universe_id}/ordered-data-stores/{ordered_data_store_id}/scopes/{scope_id}/entries/{entry_id}", summary: "Get Ordered Data Store Entry", params: &["universe_id", "ordered_data_store_id", "scope_id", "entry_id"] },
    Operation { name: "get-place", method: "GET", path: "/cloud/v2/universes/{universe_id}/places/{place_id}", summary: "Get Place", params: &["universe_id", "place_id"] },
    Operation { name: "get-public-key", method: "GET", path: "/cloud/v2/universes/{universeId}/secrets/public-key", summary: "Get Public Key", params: &["universeId"] },
    Operation { name: "get-subscription", method: "GET", path: "/cloud/v2/universes/{universe_id}/subscription-products/{subscription_product_id}/subscriptions/{subscription_id}", summary: "Get Subscription", params: &["universe_id", "subscription_product_id", "subscription_id"] },
    Operation { name: "get-table-information-by-the-assetid-of-the-table", method: "GET", path: "/legacy-localization-tables/v1/localization-table/tables/{assetId}", summary: "Get table information by the assetId of the table.", params: &["assetId"] },
    Operation { name: "get-table-information-by-the-id-of-the-table", method: "GET", path: "/legacy-localization-tables/v1/localization-table/tables/{tableId}", summary: "Get table information by the id of the table.", params: &["tableId"] },
    Operation { name: "get-the-automatic-translation-status-of-supported-languages-for-a-game", method: "GET", path: "/legacy-game-internationalization/v1/supported-languages/games/{gameId}/automatic-translation-status", summary: "Get the automatic translation status of supported languages for a game.", params: &["gameId"] },
    Operation { name: "get-the-result-of-an-asset-creation-or-update", method: "GET", path: "/assets/v1/operations/{operationId}", summary: "Get the result of an asset creation or update.", params: &["operationId"] },
    Operation { name: "get-universe", method: "GET", path: "/cloud/v2/universes/{universe_id}", summary: "Get Universe", params: &["universe_id"] },
    Operation { name: "get-universedisplayinfo-automatic-translation-settings", method: "GET", path: "/legacy-game-internationalization/v1/supported-languages/games/{gameId}/universe-display-info-automatic-translation-settings", summary: "Get UniverseDisplayInfo automatic translation settings.", params: &["gameId"] },
    Operation { name: "get-update-instance-operation", method: "GET", path: "/cloud/v2/universes/{universe_id}/places/{place_id}/instances/{instance_id}/operations/{operation_id}", summary: "Get Update Instance Operation", params: &["universe_id", "place_id", "instance_id", "operation_id"] },
    Operation { name: "get-user", method: "GET", path: "/cloud/v2/users/{user_id}", summary: "Get User", params: &["user_id"] },
    Operation { name: "get-user-restriction", method: "GET", path: "/cloud/v2/universes/{universe_id}/places/{place_id}/user-restrictions/{user_restriction_id}", summary: "Get User Restriction", params: &["universe_id", "place_id", "user_restriction_id"] },
    Operation { name: "get-user-restriction-by-universe-id-user-restrictions-by-user-restriction-id", method: "GET", path: "/cloud/v2/universes/{universe_id}/user-restrictions/{user_restriction_id}", summary: "Get User Restriction", params: &["universe_id", "user_restriction_id"] },
    Operation { name: "get-user-thumbnail-generation-operation", method: "GET", path: "/cloud/v2/users/{user_id}/operations/{operation_id}", summary: "Get User Thumbnail Generation Operation", params: &["user_id", "operation_id"] },
    Operation { name: "gets-a-batch-of-entries-for-a-table", method: "GET", path: "/legacy-localization-tables/v1/localization-table/tables/{tableId}/entries", summary: "Gets a batch of entries for a table.", params: &["tableId"] },
    Operation { name: "gets-a-list-of-groups-that-a-user-can-manage", method: "GET", path: "/legacy-develop/v1/user/groups/canmanage", summary: "Gets a list of Groups that a user can manage.", params: &[] },
    Operation { name: "gets-all-the-followings-between-a-user-with-userid-and-universes", method: "GET", path: "/legacy-followings/v1/users/{userId}/universes", summary: "Gets all the followings between a user with userId and universes", params: &["userId"] },
    Operation { name: "gets-all-universes-followed-by-a-user", method: "GET", path: "/legacy-followings/v2/users/{userId}/universes", summary: "Gets all universes followed by a user.", params: &["userId"] },
    Operation { name: "gets-available-filter-options-for-game-servers", method: "GET", path: "/server-management/v1/universes/{universeId}/places/{placeId}/game-servers:filter-options", summary: "Gets available filter options for game servers.", params: &["universeId", "placeId"] },
    Operation { name: "gets-full-config-with-metadata", method: "GET", path: "/creator-configs-public-api/v1/configs/universes/{universeId}/repositories/{repository}/full", summary: "Gets full config with metadata.", params: &["universeId", "repository"] },
    Operation { name: "gets-group-policy-info-used-for-compliance", method: "POST", path: "/legacy-groups/v1/groups/policies", summary: "Gets group policy info used for compliance.", params: &[] },
    Operation { name: "gets-groups-that-the-authenticated-user-has-requested-to-join", method: "GET", path: "/legacy-groups/v1/user/groups/pending", summary: "Gets groups that the authenticated user has requested to join", params: &[] },
    Operation { name: "gets-published-config-values-without-metadata", method: "GET", path: "/creator-configs-public-api/v1/configs/universes/{universeId}/repositories/{repository}", summary: "Gets published config values without metadata.", params: &["universeId", "repository"] },
    Operation { name: "gets-saves-from-a-collection", method: "GET", path: "/toolbox-service/v1/saves", summary: "Gets saves from a collection.", params: &[] },
    Operation { name: "gets-teamcreate-settings-for-an-roblox-platform-universes-iuniverse", method: "GET", path: "/legacy-develop/v1/universes/{universeId}/teamcreate", summary: "Gets TeamCreate settings for an Roblox.Platform.Universes.IUniverse.", params: &["universeId"] },
    Operation { name: "gets-teamcreate-settings-for-multiple-universes-specified-by-ids", method: "GET", path: "/legacy-develop/v1/universes/multiget/teamcreate", summary: "Gets TeamCreate settings for multiple universes specified by Ids", params: &[] },
    Operation { name: "gets-the-current-draft", method: "GET", path: "/creator-configs-public-api/v1/configs/universes/{universeId}/repositories/{repository}/draft", summary: "Gets the current draft.", params: &["universeId", "repository"] },
    Operation { name: "gets-the-group-s-audit-log", method: "GET", path: "/legacy-groups/v1/groups/{groupId}/audit-log", summary: "Gets the Group's audit log", params: &["groupId"] },
    Operation { name: "gets-the-group-s-settings", method: "GET", path: "/legacy-groups/v1/groups/{groupId}/settings", summary: "Gets the Group's settings", params: &["groupId"] },
    Operation { name: "gets-the-history-for-name-or-description-in-a-provided-language", method: "POST", path: "/legacy-game-internationalization/v1/name-description/games/translation-history", summary: "Gets the history for name or description in a provided language.", params: &[] },
    Operation { name: "gets-the-number-of-entries-in-the-specified-table", method: "GET", path: "/legacy-localization-tables/v1/localization-table/tables/{tableId}/entry-count", summary: "Gets the number of entries in the specified table", params: &["tableId"] },
    Operation { name: "gets-the-status-of-a-following-relationship-between-a-user-and-a-universe", method: "GET", path: "/legacy-followings/v1/users/{userId}/universes/{universeId}/status", summary: "Gets the status of a following relationship between a user and a universe.", params: &["userId", "universeId"] },
    Operation { name: "gets-the-translation-history-for-each-entry-passed-in", method: "POST", path: "/legacy-localization-tables/v1/localization-table/tables/{tableId}/entries/translation-history", summary: "Gets the translation history for each entry passed in.", params: &["tableId"] },
    Operation { name: "grant-a-subject-permission-to-multiple-assets-authorization-is-required-to-grant-permissions-to-the-subject-and-asset-ids-in-the-request", method: "PATCH", path: "/asset-permissions-api/v1/assets/permissions", summary: "Grant a subject permission to multiple assets.
            
Authorization is required to grant permissions to the subject and asset IDs in the request.", params: &[] },
    Operation { name: "increment-data-store-entry", method: "POST", path: "/cloud/v2/universes/{universe_id}/data-stores/{data_store_id}/entries/{entry_id}:increment", summary: "Increment Data Store Entry", params: &["universe_id", "data_store_id", "entry_id"] },
    Operation { name: "increment-data-store-entry-by-scope-id-entries-by-entry-id-increment", method: "POST", path: "/cloud/v2/universes/{universe_id}/data-stores/{data_store_id}/scopes/{scope_id}/entries/{entry_id}:increment", summary: "Increment Data Store Entry", params: &["universe_id", "data_store_id", "scope_id", "entry_id"] },
    Operation { name: "increment-entry", method: "POST", path: "/datastores/v1/universes/{universeId}/standard-datastores/datastore/entries/entry/increment", summary: "Increment entry", params: &["universeId"] },
    Operation { name: "increment-ordered-data-store-entry", method: "POST", path: "/cloud/v2/universes/{universe_id}/ordered-data-stores/{ordered_data_store_id}/scopes/{scope_id}/entries/{entry_id}:increment", summary: "Increment Ordered Data Store Entry", params: &["universe_id", "ordered_data_store_id", "scope_id", "entry_id"] },
    Operation { name: "launch-a-game-server-restart-for-a-universe", method: "POST", path: "/server-management/v1/universes/{universeId}/restarts", summary: "Launch a game server restart for a universe.", params: &["universeId"] },
    Operation { name: "list-asset-quotas", method: "GET", path: "/cloud/v2/users/{user_id}/asset-quotas", summary: "List Asset Quotas", params: &["user_id"] },
    Operation { name: "list-asset-versions-of-an-asset", method: "GET", path: "/assets/v1/assets/{assetId}/versions", summary: "List Asset Versions of an Asset", params: &["assetId"] },
    Operation { name: "list-data-store-entries", method: "GET", path: "/cloud/v2/universes/{universe_id}/data-stores/{data_store_id}/entries", summary: "List Data Store Entries", params: &["universe_id", "data_store_id"] },
    Operation { name: "list-data-store-entries-scopes-by-scope-id-entries", method: "GET", path: "/cloud/v2/universes/{universe_id}/data-stores/{data_store_id}/scopes/{scope_id}/entries", summary: "List Data Store Entries", params: &["universe_id", "data_store_id", "scope_id"] },
    Operation { name: "list-data-store-entry-revisions", method: "GET", path: "/cloud/v2/universes/{universe_id}/data-stores/{data_store_id}/entries/{entry_id}:listRevisions", summary: "List Data Store Entry Revisions", params: &["universe_id", "data_store_id", "entry_id"] },
    Operation { name: "list-data-store-entry-revisions-by-scope-id-entries-by-entry-id-listrevisions", method: "GET", path: "/cloud/v2/universes/{universe_id}/data-stores/{data_store_id}/scopes/{scope_id}/entries/{entry_id}:listRevisions", summary: "List Data Store Entry Revisions", params: &["universe_id", "data_store_id", "scope_id", "entry_id"] },
    Operation { name: "list-data-stores", method: "GET", path: "/cloud/v2/universes/{universe_id}/data-stores", summary: "List Data Stores", params: &["universe_id"] },
    Operation { name: "list-data-stores-in-an-experience", method: "GET", path: "/datastores/v1/universes/{universeId}/standard-datastores", summary: "List data stores in an experience", params: &["universeId"] },
    Operation { name: "list-developer-products-by-universe-with-configuration-details", method: "GET", path: "/developer-products/v2/universes/{universeId}/developer-products/creator", summary: "List developer products by universe with configuration details", params: &["universeId"] },
    Operation { name: "list-entries", method: "GET", path: "/datastores/v1/universes/{universeId}/standard-datastores/datastore/entries", summary: "List entries", params: &["universeId"] },
    Operation { name: "list-entry-versions", method: "GET", path: "/datastores/v1/universes/{universeId}/standard-datastores/datastore/entries/entry/versions", summary: "List entry versions", params: &["universeId"] },
    Operation { name: "list-game-passes-by-universe-with-configuration-details", method: "GET", path: "/game-passes/v1/universes/{universeId}/game-passes/creator", summary: "List game passes by universe with configuration details", params: &["universeId"] },
    Operation { name: "list-group-forum-categories", method: "GET", path: "/cloud/v2/groups/{group_id}/forum-categories", summary: "List Group Forum Categories", params: &["group_id"] },
    Operation { name: "list-group-forum-comments", method: "GET", path: "/cloud/v2/groups/{group_id}/forum-categories/{forum_category_id}/posts/{post_id}/comments", summary: "List Group Forum Comments", params: &["group_id", "forum_category_id", "post_id"] },
    Operation { name: "list-group-forum-posts", method: "GET", path: "/cloud/v2/groups/{group_id}/forum-categories/{forum_category_id}/posts", summary: "List Group Forum Posts", params: &["group_id", "forum_category_id"] },
    Operation { name: "list-group-join-requests", method: "GET", path: "/cloud/v2/groups/{group_id}/join-requests", summary: "List Group Join Requests", params: &["group_id"] },
    Operation { name: "list-group-memberships", method: "GET", path: "/cloud/v2/groups/{group_id}/memberships", summary: "List Group Memberships", params: &["group_id"] },
    Operation { name: "list-group-roles", method: "GET", path: "/cloud/v2/groups/{group_id}/roles", summary: "List Group Roles", params: &["group_id"] },
    Operation { name: "list-instance-children", method: "GET", path: "/cloud/v2/universes/{universe_id}/places/{place_id}/instances/{instance_id}:listChildren", summary: "List Instance Children", params: &["universe_id", "place_id", "instance_id"] },
    Operation { name: "list-inventory-items", method: "GET", path: "/cloud/v2/users/{user_id}/inventory-items", summary: "List Inventory Items", params: &["user_id"] },
    Operation { name: "list-luau-execution-session-task-logs", method: "GET", path: "/cloud/v2/universes/{universe_id}/places/{place_id}/versions/{version_id}/luau-execution-sessions/{luau_execution_session_id}/tasks/{task_id}/logs", summary: "List Luau Execution Session Task Logs", params: &["universe_id", "place_id", "version_id", "luau_execution_session_id", "task_id"] },
    Operation { name: "list-memory-store-sorted-map-items", method: "GET", path: "/cloud/v2/universes/{universe_id}/memory-store/sorted-maps/{sorted_map_id}/items", summary: "List Memory Store Sorted Map Items", params: &["universe_id", "sorted_map_id"] },
    Operation { name: "list-of-users-in-the-active-team-create-session", method: "GET", path: "/legacy-develop/v1/places/{placeId}/teamcreate/active_session/members", summary: "List of users in the active Team Create session", params: &["placeId"] },
    Operation { name: "list-ordered-data-store-entries", method: "GET", path: "/cloud/v2/universes/{universe_id}/ordered-data-stores/{ordered_data_store_id}/scopes/{scope_id}/entries", summary: "List Ordered Data Store Entries", params: &["universe_id", "ordered_data_store_id", "scope_id"] },
    Operation { name: "list-restart-statuses-for-a-universe", method: "GET", path: "/server-management/v1/universes/{universeId}/restarts", summary: "List restart statuses for a universe.", params: &["universeId"] },
    Operation { name: "list-secrets", method: "GET", path: "/cloud/v2/universes/{universeId}/secrets", summary: "List Secrets", params: &["universeId"] },
    Operation { name: "list-user-restriction-logs", method: "GET", path: "/cloud/v2/universes/{universe_id}/user-restrictions:listLogs", summary: "List User Restriction Logs", params: &["universe_id"] },
    Operation { name: "list-user-restrictions", method: "GET", path: "/cloud/v2/universes/{universe_id}/places/{place_id}/user-restrictions", summary: "List User Restrictions", params: &["universe_id", "place_id"] },
    Operation { name: "list-user-restrictions-universes-by-universe-id-user-restrictions", method: "GET", path: "/cloud/v2/universes/{universe_id}/user-restrictions", summary: "List User Restrictions", params: &["universe_id"] },
    Operation { name: "lists-game-servers-for-a-specific-place-version", method: "GET", path: "/server-management/v1/universes/{universeId}/places/{placeId}/versions/{versionNumber}/game-servers", summary: "Lists game servers for a specific place version.", params: &["universeId", "placeId", "versionNumber"] },
    Operation { name: "lists-revision-history", method: "GET", path: "/creator-configs-public-api/v1/configs/universes/{universeId}/repositories/{repository}/revisions", summary: "Lists revision history.", params: &["universeId", "repository"] },
    Operation { name: "metadata-for-autolocalization-configuration", method: "GET", path: "/legacy-localization-tables/v1/autolocalization/metadata", summary: "Metadata for AutoLocalization Configuration", params: &[] },
    Operation { name: "ordereddatastores-createentry", method: "POST", path: "/ordered-data-stores/v1/universes/{universeId}/orderedDataStores/{orderedDataStore}/scopes/{scope}/entries", summary: "OrderedDataStores_CreateEntry", params: &["universeId", "orderedDataStore", "scope"] },
    Operation { name: "ordereddatastores-deleteentry", method: "DELETE", path: "/ordered-data-stores/v1/universes/{universeId}/orderedDataStores/{orderedDataStore}/scopes/{scope}/entries/{entry}", summary: "OrderedDataStores_DeleteEntry", params: &["universeId", "orderedDataStore", "scope", "entry"] },
    Operation { name: "ordereddatastores-getentry", method: "GET", path: "/ordered-data-stores/v1/universes/{universeId}/orderedDataStores/{orderedDataStore}/scopes/{scope}/entries/{entry}", summary: "OrderedDataStores_GetEntry", params: &["universeId", "orderedDataStore", "scope", "entry"] },
    Operation { name: "ordereddatastores-incremententry", method: "POST", path: "/ordered-data-stores/v1/universes/{universeId}/orderedDataStores/{orderedDataStore}/scopes/{scope}/entries/{entry}:increment", summary: "OrderedDataStores_IncrementEntry", params: &["universeId", "orderedDataStore", "scope", "entry"] },
    Operation { name: "ordereddatastores-listentries", method: "GET", path: "/ordered-data-stores/v1/universes/{universeId}/orderedDataStores/{orderedDataStore}/scopes/{scope}/entries", summary: "OrderedDataStores_ListEntries", params: &["universeId", "orderedDataStore", "scope"] },
    Operation { name: "ordereddatastores-updateentry", method: "PATCH", path: "/ordered-data-stores/v1/universes/{universeId}/orderedDataStores/{orderedDataStore}/scopes/{scope}/entries/{entry}", summary: "OrderedDataStores_UpdateEntry", params: &["universeId", "orderedDataStore", "scope", "entry"] },
    Operation { name: "orders-the-specified-image-ids-for-the-game-thumbnails", method: "POST", path: "/legacy-game-internationalization/v1/game-thumbnails/games/{gameId}/language-codes/{languageCode}/images/order", summary: "Orders the specified image Ids for the game thumbnails.", params: &["gameId", "languageCode"] },
    Operation { name: "overwrites-a-badge-icon-with-a-new-one", method: "POST", path: "/legacy-publish/v1/badges/{badgeId}/icon", summary: "Overwrites a badge icon with a new one.", params: &["badgeId"] },
    Operation { name: "overwrites-the-entire-draft-with-the-request-payload", method: "PUT", path: "/creator-configs-public-api/v1/configs/universes/{universeId}/repositories/{repository}/draft:overwrite", summary: "Overwrites the entire draft with the request payload.", params: &["universeId", "repository"] },
    Operation { name: "partially-updates-the-draft", method: "PATCH", path: "/creator-configs-public-api/v1/configs/universes/{universeId}/repositories/{repository}/draft", summary: "Partially updates the draft.", params: &["universeId", "repository"] },
    Operation { name: "places-createplaceversionapikey", method: "POST", path: "/universes/v1/{universeId}/places/{placeId}/versions", summary: "Places_CreatePlaceVersionApiKey", params: &["universeId", "placeId"] },
    Operation { name: "post-legacy-localization-tables-v1-autolocalization-games-gameid-autolocalizationtable", method: "POST", path: "/legacy-localization-tables/v1/autolocalization/games/{gameId}/autolocalizationtable", summary: "post /legacy-localization-tables/v1/autolocalization/games/{gameId}/autolocalizationtable", params: &["gameId"] },
    Operation { name: "publish-a-cross-server-message-to-a-universe", method: "POST", path: "/messaging-service/v1/universes/{universeId}/topics/{topic}", summary: "Publish a cross-server message to a universe", params: &["universeId", "topic"] },
    Operation { name: "publish-universe-message", method: "POST", path: "/cloud/v2/universes/{universe_id}:publishMessage", summary: "Publish Universe Message", params: &["universe_id"] },
    Operation { name: "publishes-draft-changes", method: "POST", path: "/creator-configs-public-api/v1/configs/universes/{universeId}/repositories/{repository}/publish", summary: "Publishes draft changes.", params: &["universeId", "repository"] },
    Operation { name: "read-memory-store-queue-items", method: "GET", path: "/cloud/v2/universes/{universe_id}/memory-store/queues/{queue_id}/items:read", summary: "Read Memory Store Queue Items", params: &["universe_id", "queue_id"] },
    Operation { name: "removes-a-user-from-a-teamcreate-permissions-list", method: "DELETE", path: "/legacy-develop/v1/universes/{universeId}/teamcreate/memberships", summary: "Removes a user from a TeamCreate permissions list.", params: &["universeId"] },
    Operation { name: "resets-deletes-the-current-draft", method: "DELETE", path: "/creator-configs-public-api/v1/configs/universes/{universeId}/repositories/{repository}/draft", summary: "Resets (deletes) the current draft.", params: &["universeId", "repository"] },
    Operation { name: "restart-universe-servers", method: "POST", path: "/cloud/v2/universes/{universe_id}:restartServers", summary: "Restart Universe Servers", params: &["universe_id"] },
    Operation { name: "restores-an-archived-asset", method: "POST", path: "/assets/v1/assets/{assetId}:restore", summary: "Restores an archived asset.", params: &["assetId"] },
    Operation { name: "retrieve-specific-asset-metadata-include-the-readmask-parameter-for-additional-asset-metadata", method: "GET", path: "/assets/v1/assets/{assetId}", summary: "Retrieve specific asset metadata. Include the `readMask` parameter for additional asset metadata.", params: &["assetId"] },
    Operation { name: "retrieves-an-asset-by-its-id-and-version-number-with-opencloud-auth", method: "GET", path: "/asset-delivery-api/v1/assetId/{assetId}/version/{versionNumber}", summary: "Retrieves an asset by its ID and version number with OpenCloud auth.", params: &["assetId", "versionNumber"] },
    Operation { name: "retrieves-an-asset-by-its-id-with-opencloud-auth", method: "GET", path: "/asset-delivery-api/v1/assetId/{assetId}", summary: "Retrieves an asset by its ID with OpenCloud auth.", params: &["assetId"] },
    Operation { name: "returns-an-array-of-granted-and-declined-permissions-related-to-the-universes-with-the-ids-in-ids-for-the-authenticated-user", method: "GET", path: "/legacy-develop/v1/universes/multiget/permissions", summary: "Returns an array of granted and declined permissions related to the universes with the ids in ids for the authenticated user.", params: &[] },
    Operation { name: "returns-list-of-granted-and-declined-permissions-related-to-the-universe-with-the-id-universeid-for-authenticated-user", method: "GET", path: "/legacy-develop/v1/universes/{universeId}/permissions", summary: "Returns list of granted and declined permissions related to the universe with the id universeId for authenticated user", params: &["universeId"] },
    Operation { name: "rollback-an-asset-to-a-previous-version", method: "POST", path: "/assets/v1/assets/{assetId}/versions:rollback", summary: "Rollback an asset to a previous version.", params: &["assetId"] },
    Operation { name: "search-creator-store-assets", method: "GET", path: "/toolbox-service/v2/assets:search", summary: "Search Creator Store Assets", params: &[] },
    Operation { name: "search-creator-store-assets-toolbox-service-v2-assets-search", method: "POST", path: "/toolbox-service/v2/assets:search", summary: "Search Creator Store Assets", params: &[] },
    Operation { name: "set-entry", method: "POST", path: "/datastores/v1/universes/{universeId}/standard-datastores/datastore/entries/entry", summary: "Set entry.", params: &["universeId"] },
    Operation { name: "sets-a-game-s-auto-localization-related-settings", method: "PATCH", path: "/legacy-localization-tables/v1/autolocalization/games/{gameId}/settings", summary: "Sets a game's auto-localization related settings", params: &["gameId"] },
    Operation { name: "sets-group-status", method: "PATCH", path: "/legacy-groups/v1/groups/{groupId}/status", summary: "Sets group status", params: &["groupId"] },
    Operation { name: "sets-the-source-language-of-a-game", method: "PATCH", path: "/legacy-game-internationalization/v1/source-language/games/{gameId}", summary: "Sets the source language of a game", params: &["gameId"] },
    Operation { name: "snapshot-data-stores", method: "POST", path: "/cloud/v2/universes/{universe_id}/data-stores:snapshot", summary: "Snapshot Data Stores", params: &["universe_id"] },
    Operation { name: "stages-a-revert-to-a-previous-revision", method: "POST", path: "/creator-configs-public-api/v1/configs/universes/{universeId}/repositories/{repository}/revisions/{revisionId}/restore", summary: "Stages a revert to a previous revision.", params: &["universeId", "repository", "revisionId"] },
    Operation { name: "thumbnails-assets", method: "GET", path: "/v1/assets-thumbnail-3d", summary: "Thumbnails assets.", params: &[] },
    Operation { name: "translate-text", method: "POST", path: "/cloud/v2/universes/{universe_id}:translateText", summary: "Translate Text", params: &["universe_id"] },
    Operation { name: "unassign-role-group-membership", method: "POST", path: "/cloud/v2/groups/{group_id}/memberships/{membership_id}:unassignRole", summary: "Unassign Role Group Membership", params: &["group_id", "membership_id"] },
    Operation { name: "undelete-data-store", method: "POST", path: "/cloud/v2/universes/{universe_id}/data-stores/{data_store_id}:undelete", summary: "Undelete Data Store", params: &["universe_id", "data_store_id"] },
    Operation { name: "update-a-badge-s-icon", method: "POST", path: "/legacy-game-internationalization/v1/badges/{badgeId}/icons/language-codes/{languageCode}", summary: "Update a badge's icon", params: &["badgeId", "languageCode"] },
    Operation { name: "update-a-developer-product-s-icon", method: "POST", path: "/legacy-game-internationalization/v1/developer-products/{developerProductId}/icons/language-codes/{languageCode}", summary: "Update a developer product's icon", params: &["developerProductId", "languageCode"] },
    Operation { name: "update-a-game-pass-s-icon", method: "POST", path: "/legacy-game-internationalization/v1/game-passes/{gamePassId}/icons/language-codes/{languageCode}", summary: "Update a game pass's icon", params: &["gamePassId", "languageCode"] },
    Operation { name: "update-a-game-s-icon", method: "POST", path: "/legacy-game-internationalization/v1/game-icon/games/{gameId}/language-codes/{languageCode}", summary: "Update a game's icon", params: &["gameId", "languageCode"] },
    Operation { name: "update-creator-store-product", method: "PATCH", path: "/cloud/v2/creator-store-products/{creator_store_product_id}", summary: "Update Creator Store Product", params: &["creator_store_product_id"] },
    Operation { name: "update-data-store-entry", method: "PATCH", path: "/cloud/v2/universes/{universe_id}/data-stores/{data_store_id}/entries/{entry_id}", summary: "Update Data Store Entry", params: &["universe_id", "data_store_id", "entry_id"] },
    Operation { name: "update-data-store-entry-by-scope-id-entries-by-entry-id", method: "PATCH", path: "/cloud/v2/universes/{universe_id}/data-stores/{data_store_id}/scopes/{scope_id}/entries/{entry_id}", summary: "Update Data Store Entry", params: &["universe_id", "data_store_id", "scope_id", "entry_id"] },
    Operation { name: "update-developer-product", method: "PATCH", path: "/developer-products/v2/universes/{universeId}/developer-products/{productId}", summary: "Update developer product", params: &["universeId", "productId"] },
    Operation { name: "update-game-pass", method: "PATCH", path: "/game-passes/v1/universes/{universeId}/game-passes/{gamePassId}", summary: "Update game pass", params: &["universeId", "gamePassId"] },
    Operation { name: "update-group-membership", method: "PATCH", path: "/cloud/v2/groups/{group_id}/memberships/{membership_id}", summary: "Update Group Membership", params: &["group_id", "membership_id"] },
    Operation { name: "update-instance", method: "PATCH", path: "/cloud/v2/universes/{universe_id}/places/{place_id}/instances/{instance_id}", summary: "Update Instance", params: &["universe_id", "place_id", "instance_id"] },
    Operation { name: "update-localized-description-of-a-badge", method: "PATCH", path: "/legacy-game-internationalization/v1/badges/{badgeId}/description/language-codes/{languageCode}", summary: "Update localized description of a badge", params: &["badgeId", "languageCode"] },
    Operation { name: "update-localized-description-of-a-developer-product", method: "PATCH", path: "/legacy-game-internationalization/v1/developer-products/{developerProductId}/description/language-codes/{languageCode}", summary: "Update localized description of a developer product", params: &["developerProductId", "languageCode"] },
    Operation { name: "update-localized-description-of-a-game-pass", method: "PATCH", path: "/legacy-game-internationalization/v1/game-passes/{gamePassId}/description/language-codes/{languageCode}", summary: "Update localized description of a game pass", params: &["gamePassId", "languageCode"] },
    Operation { name: "update-localized-name-and-description-of-a-badge", method: "PATCH", path: "/legacy-game-internationalization/v1/badges/{badgeId}/name-description/language-codes/{languageCode}", summary: "Update localized name and description of a badge", params: &["badgeId", "languageCode"] },
    Operation { name: "update-localized-name-and-description-of-a-developer-product", method: "PATCH", path: "/legacy-game-internationalization/v1/developer-products/{developerProductId}/name-description/language-codes/{languageCode}", summary: "Update localized name and description of a developer product", params: &["developerProductId", "languageCode"] },
    Operation { name: "update-localized-name-and-description-of-a-game-pass", method: "PATCH", path: "/legacy-game-internationalization/v1/game-passes/{gamePassId}/name-description/language-codes/{languageCode}", summary: "Update localized name and description of a game pass", params: &["gamePassId", "languageCode"] },
    Operation { name: "update-localized-name-of-a-badge", method: "PATCH", path: "/legacy-game-internationalization/v1/badges/{badgeId}/name/language-codes/{languageCode}", summary: "Update localized name of a badge", params: &["badgeId", "languageCode"] },
    Operation { name: "update-localized-name-of-a-developer-product", method: "PATCH", path: "/legacy-game-internationalization/v1/developer-products/{developerProductId}/name/language-codes/{languageCode}", summary: "Update localized name of a developer product", params: &["developerProductId", "languageCode"] },
    Operation { name: "update-localized-name-of-a-game-pass", method: "PATCH", path: "/legacy-game-internationalization/v1/game-passes/{gamePassId}/name/language-codes/{languageCode}", summary: "Update localized name of a game pass", params: &["gamePassId", "languageCode"] },
    Operation { name: "update-memory-store-sorted-map-item", method: "PATCH", path: "/cloud/v2/universes/{universe_id}/memory-store/sorted-maps/{sorted_map_id}/items/{item_id}", summary: "Update Memory Store Sorted Map Item", params: &["universe_id", "sorted_map_id", "item_id"] },
    Operation { name: "update-ordered-data-store-entry", method: "PATCH", path: "/cloud/v2/universes/{universe_id}/ordered-data-stores/{ordered_data_store_id}/scopes/{scope_id}/entries/{entry_id}", summary: "Update Ordered Data Store Entry", params: &["universe_id", "ordered_data_store_id", "scope_id", "entry_id"] },
    Operation { name: "update-place", method: "PATCH", path: "/cloud/v2/universes/{universe_id}/places/{place_id}", summary: "Update Place", params: &["universe_id", "place_id"] },
    Operation { name: "update-secret", method: "PATCH", path: "/cloud/v2/universes/{universeId}/secrets/{secretId}", summary: "Update Secret", params: &["universeId", "secretId"] },
    Operation { name: "update-the-switch-which-controls-if-the-universedisplayinformation-should-be-automatically-translated", method: "PATCH", path: "/legacy-game-internationalization/v1/supported-languages/games/{gameId}/languages/{languageCode}/universe-display-info-automatic-translation-settings", summary: "Update the switch which controls if the UniverseDisplayInformation should be automatically translated.", params: &["gameId", "languageCode"] },
    Operation { name: "update-universe", method: "PATCH", path: "/cloud/v2/universes/{universe_id}", summary: "Update Universe", params: &["universe_id"] },
    Operation { name: "update-user-restriction", method: "PATCH", path: "/cloud/v2/universes/{universe_id}/places/{place_id}/user-restrictions/{user_restriction_id}", summary: "Update User Restriction", params: &["universe_id", "place_id", "user_restriction_id"] },
    Operation { name: "update-user-restriction-by-universe-id-user-restrictions-by-user-restriction-id", method: "PATCH", path: "/cloud/v2/universes/{universe_id}/user-restrictions/{user_restriction_id}", summary: "Update User Restriction", params: &["universe_id", "user_restriction_id"] },
    Operation { name: "updates-a-game-s-name-and-or-description-in-multiple-languages", method: "PATCH", path: "/legacy-game-internationalization/v1/name-description/games/{gameId}", summary: "Updates a game's name and/or description in multiple languages.", params: &["gameId"] },
    Operation { name: "updates-an-asset-with-provided-content-and-metadata", method: "PATCH", path: "/assets/v1/assets/{assetId}", summary: "Updates an asset with provided content and metadata.", params: &["assetId"] },
    Operation { name: "updates-badge-configuration", method: "PATCH", path: "/legacy-badges/v1/badges/{badgeId}", summary: "Updates badge configuration.", params: &["badgeId"] },
    Operation { name: "updates-the-game-thumbnail-alt-text", method: "POST", path: "/legacy-game-internationalization/v1/game-thumbnails/games/{gameId}/language-codes/{languageCode}/alt-text", summary: "Updates the game thumbnail alt text.", params: &["gameId", "languageCode"] },
    Operation { name: "updates-the-group-s-settings", method: "PATCH", path: "/legacy-groups/v1/groups/{groupId}/notification-preference", summary: "Updates the group's settings", params: &["groupId"] },
    Operation { name: "updates-the-group-s-settings-groups-by-groupid-settings", method: "PATCH", path: "/legacy-groups/v1/groups/{groupId}/settings", summary: "Updates the group's settings", params: &["groupId"] },
    Operation { name: "updates-the-groups-description", method: "PATCH", path: "/legacy-groups/v1/groups/{groupId}/description", summary: "Updates the groups description", params: &["groupId"] },
    Operation { name: "updates-the-tables-contents-based-on-what-is-provided", method: "PATCH", path: "/legacy-localization-tables/v1/localization-table/tables/{tableId}", summary: "Updates the tables contents based on what is provided.", params: &["tableId"] },
    Operation { name: "uploads-the-game-thumbnail", method: "POST", path: "/legacy-game-internationalization/v1/game-thumbnails/games/{gameId}/language-codes/{languageCode}/image", summary: "Uploads the game thumbnail.", params: &["gameId", "languageCode"] }
];

#[derive(Debug, Subcommand)]
pub enum CloudCommands {
    /// List all generated API-key operations
    Operations {
        /// Filter operations by name, summary, method, or path
        #[clap(short, long, value_parser)]
        filter: Option<String>,
    },

    /// Call a generated API-key operation by name
    Call {
        /// Operation name from `rbxcloud cloud operations`
        operation: String,

        /// Path parameter as key=value. Repeat for multiple params.
        #[clap(long = "param", value_parser)]
        params: Vec<String>,

        /// Query parameter as key=value. Repeat for multiple params.
        #[clap(short, long, value_parser)]
        query: Vec<String>,

        /// JSON request body
        #[clap(short, long, value_parser)]
        body: Option<String>,

        /// Pretty-print JSON response
        #[clap(short, long, value_parser, default_value_t = false)]
        pretty: bool,

        /// Roblox Open Cloud API Key
        #[clap(short, long, value_parser, env = "RBXCLOUD_API_KEY")]
        api_key: String,
    },
}

#[derive(Debug, Args)]
pub struct Cloud {
    #[clap(subcommand)]
    command: CloudCommands,
}

fn parse_pairs(values: Vec<String>, field: &str) -> anyhow::Result<Vec<(String, String)>> {
    values
        .into_iter()
        .map(|item| {
            let Some((key, value)) = item.split_once('=') else {
                anyhow::bail!("{} must be key=value: {}", field, item);
            };
            Ok((key.to_string(), value.to_string()))
        })
        .collect()
}

fn fill_path(path: &str, params: &[(String, String)], required: &[&str]) -> anyhow::Result<String> {
    let mut out = path.to_string();
    for required_param in required {
        let Some((_, value)) = params.iter().find(|(key, _)| key == required_param) else {
            anyhow::bail!(
                "missing --param {}=... for endpoint {}",
                required_param,
                path
            );
        };
        out = out.replace(&format!("{{{}}}", required_param), value);
    }
    Ok(out)
}

fn format_response(text: String, pretty: bool) -> anyhow::Result<String> {
    if pretty {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
            return Ok(serde_json::to_string_pretty(&json)?);
        }
    }
    Ok(text)
}

impl Cloud {
    pub async fn run(self) -> anyhow::Result<Option<String>> {
        match self.command {
            CloudCommands::Operations { filter } => {
                let filter = filter.map(|value| value.to_lowercase());
                let mut lines = Vec::new();
                for op in OPERATIONS {
                    if let Some(filter) = &filter {
                        let haystack =
                            format!("{} {} {} {}", op.name, op.method, op.path, op.summary)
                                .to_lowercase();
                        if !haystack.contains(filter) {
                            continue;
                        }
                    }
                    let params = if op.params.is_empty() {
                        String::new()
                    } else {
                        format!(" params=[{}]", op.params.join(","))
                    };
                    lines.push(format!(
                        "{:<62} {:<6} {}{}\n  {}",
                        op.name, op.method, op.path, params, op.summary
                    ));
                }
                Ok(Some(lines.join("\n")))
            }
            CloudCommands::Call {
                operation,
                params,
                query,
                body,
                pretty,
                api_key,
            } => {
                let Some(op) = OPERATIONS
                    .iter()
                    .find(|candidate| candidate.name == operation)
                else {
                    let similar: Vec<&str> = OPERATIONS
                        .iter()
                        .filter(|candidate| candidate.name.contains(&operation))
                        .take(10)
                        .map(|candidate| candidate.name)
                        .collect();
                    anyhow::bail!(
                        "unknown operation '{}'. Similar: {}. Run `rbxcloud cloud operations`.",
                        operation,
                        similar.join(", ")
                    );
                };

                let params = parse_pairs(params, "--param")?;
                let query = parse_pairs(query, "--query")?;
                let path = fill_path(op.path, &params, op.params)?;
                let url = format!("https://apis.roblox.com{}", path);
                let method = op.method.parse::<Method>()?;
                let client = reqwest::Client::new();
                let mut request = client
                    .request(method, url)
                    .header("x-api-key", api_key)
                    .header("Content-Type", "application/json");
                if !query.is_empty() {
                    request = request.query(&query);
                }
                if let Some(body) = body {
                    let json_body: serde_json::Value = serde_json::from_str(&body)?;
                    request = request.json(&json_body);
                }

                let response = request.send().await?;
                let status = response.status();
                let text = response.text().await?;
                if !status.is_success() {
                    anyhow::bail!(
                        "HTTP {} calling {} {}: {}",
                        status.as_u16(),
                        op.method,
                        op.path,
                        text
                    );
                }
                Ok(Some(format_response(text, pretty)?))
            }
        }
    }
}
