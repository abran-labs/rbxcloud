# Generated Open Cloud API-Key Operations

The `cloud` command provides named operations generated from the Roblox Cloud OpenAPI spec for every endpoint that accepts `roblox-api-key` auth.

List operations:

```bash
rbxcloud cloud operations
rbxcloud cloud operations --filter group
```

Call an operation:

```bash
rbxcloud cloud call list-group-roles --param group_id=12345 --pretty
rbxcloud cloud call assign-role-group-membership --param group_id=12345 --param membership_id=156 --body '{"role":"groups/12345/roles/98765"}' --pretty
```

Use `--query key=value` for query parameters and `--body` for JSON request bodies.

## Operations

- `accept-group-join-request` — `POST /cloud/v2/groups/{group_id}/join-requests/{join_request_id}:accept` — Accept Group Join Request
- `activates-a-universes` — `POST /legacy-develop/v1/universes/{universeId}/activate` — Activates a universes.
- `add-or-remove-supported-languages-for-a-game` — `PATCH /legacy-game-internationalization/v1/supported-languages/games/{gameId}` — Add or remove supported languages for a game.
- `archives-the-asset` — `POST /assets/v1/assets/{assetId}:archive` — Archives the asset.
- `assign-role-group-membership` — `POST /cloud/v2/groups/{group_id}/memberships/{membership_id}:assignRole` — Assign Role Group Membership
- `bulk-deletes-saves-max-of-5000-saves-per-request` — `POST /toolbox-service/v1/saves:bulkDelete` — Bulk deletes saves.
Max of 5000 saves per request.
- `close-a-game-instance-that-is-being-used-for-team-testing` — `DELETE /legacy-develop/v2/teamtest/{placeId}` — Close a game instance that is being used for team testing
- `create-creator-store-product` — `POST /cloud/v2/creator-store-products` — Create Creator Store Product
- `create-data-store-entry` — `POST /cloud/v2/universes/{universe_id}/data-stores/{data_store_id}/entries` — Create Data Store Entry
- `create-data-store-entry-scopes-by-scope-id-entries` — `POST /cloud/v2/universes/{universe_id}/data-stores/{data_store_id}/scopes/{scope_id}/entries` — Create Data Store Entry
- `create-developer-product` — `POST /developer-products/v2/universes/{universeId}/developer-products` — Create developer product
- `create-game-pass` — `POST /game-passes/v1/universes/{universeId}/game-passes` — Create game pass
- `create-luau-execution-session-task` — `POST /cloud/v2/universes/{universe_id}/places/{place_id}/luau-execution-session-tasks` — Create Luau Execution Session Task
- `create-luau-execution-session-task-binary-input` — `POST /cloud/v2/universes/{universe_id}/luau-execution-session-task-binary-inputs` — Create Luau Execution Session Task Binary Input
- `create-luau-execution-session-task-versions-by-version-id-luau-execution-session-tasks` — `POST /cloud/v2/universes/{universe_id}/places/{place_id}/versions/{version_id}/luau-execution-session-tasks` — Create Luau Execution Session Task
- `create-memory-store-queue-item` — `POST /cloud/v2/universes/{universe_id}/memory-store/queues/{queue_id}/items` — Create Memory Store Queue Item
- `create-memory-store-sorted-map-item` — `POST /cloud/v2/universes/{universe_id}/memory-store/sorted-maps/{sorted_map_id}/items` — Create Memory Store Sorted Map Item
- `create-ordered-data-store-entry` — `POST /cloud/v2/universes/{universe_id}/ordered-data-stores/{ordered_data_store_id}/scopes/{scope_id}/entries` — Create Ordered Data Store Entry
- `create-secret` — `POST /cloud/v2/universes/{universeId}/secrets` — Create Secret
- `create-user-notification` — `POST /cloud/v2/users/{user_id}/notifications` — Create User Notification
- `creates-a-new-badge` — `POST /legacy-badges/v1/universes/{universeId}/badges` — Creates a new badge.
- `creates-a-save` — `POST /toolbox-service/v1/saves` — Creates a save.
- `creates-an-asset-with-provided-content-and-metadata` — `POST /assets/v1/assets` — Creates an asset with provided content and metadata.
- `creates-the-following-between-a-user-with-userid-and-universe-with-universeid` — `POST /legacy-followings/v1/users/{userId}/universes/{universeId}` — Creates the following between a user with userId and universe with universeId
- `deactivates-a-universe` — `POST /legacy-develop/v1/universes/{universeId}/deactivate` — Deactivates a universe.
- `decline-group-join-request` — `POST /cloud/v2/groups/{group_id}/join-requests/{join_request_id}:decline` — Decline Group Join Request
- `delete-a-localized-icon-from-a-badge` — `DELETE /legacy-game-internationalization/v1/badges/{badgeId}/icons/language-codes/{languageCode}` — Delete a localized icon from a badge
- `delete-a-localized-icon-from-a-developer-product` — `DELETE /legacy-game-internationalization/v1/developer-products/{developerProductId}/icons/language-codes/{languageCode}` — Delete a localized icon from a developer product
- `delete-a-localized-icon-from-a-game` — `DELETE /legacy-game-internationalization/v1/game-icon/games/{gameId}/language-codes/{languageCode}` — Delete a localized icon from a game
- `delete-a-localized-icon-from-a-game-pass` — `DELETE /legacy-game-internationalization/v1/game-passes/{gamePassId}/icons/language-codes/{languageCode}` — Delete a localized icon from a game pass
- `delete-data-store` — `DELETE /cloud/v2/universes/{universe_id}/data-stores/{data_store_id}` — Delete Data Store
- `delete-data-store-entry` — `DELETE /cloud/v2/universes/{universe_id}/data-stores/{data_store_id}/entries/{entry_id}` — Delete Data Store Entry
- `delete-data-store-entry-by-scope-id-entries-by-entry-id` — `DELETE /cloud/v2/universes/{universe_id}/data-stores/{data_store_id}/scopes/{scope_id}/entries/{entry_id}` — Delete Data Store Entry
- `delete-entry` — `DELETE /datastores/v1/universes/{universeId}/standard-datastores/datastore/entries/entry` — Delete entry.
- `delete-localized-name-and-description-of-a-badge` — `DELETE /legacy-game-internationalization/v1/badges/{badgeId}/name-description/language-codes/{languageCode}` — Delete localized name and description of a badge
- `delete-localized-name-and-description-of-a-developer-product` — `DELETE /legacy-game-internationalization/v1/developer-products/{developerProductId}/name-description/language-codes/{languageCode}` — Delete localized name and description of a developer product
- `delete-localized-name-and-description-of-a-game-pass` — `DELETE /legacy-game-internationalization/v1/game-passes/{gamePassId}/name-description/language-codes/{languageCode}` — Delete localized name and description of a game pass
- `delete-memory-store-sorted-map-item` — `DELETE /cloud/v2/universes/{universe_id}/memory-store/sorted-maps/{sorted_map_id}/items/{item_id}` — Delete Memory Store Sorted Map Item
- `delete-ordered-data-store-entry` — `DELETE /cloud/v2/universes/{universe_id}/ordered-data-stores/{ordered_data_store_id}/scopes/{scope_id}/entries/{entry_id}` — Delete Ordered Data Store Entry
- `delete-secret` — `DELETE /cloud/v2/universes/{universeId}/secrets/{secretId}` — Delete Secret
- `deletes-a-save` — `DELETE /toolbox-service/v1/saves` — Deletes a save.
- `deletes-the-following-between-a-user-with-userid-and-universe-with-universeid` — `DELETE /legacy-followings/v1/users/{userId}/universes/{universeId}` — Deletes the following between a user with userId and universe with universeId
- `deletes-the-game-thumbnail` — `DELETE /legacy-game-internationalization/v1/game-thumbnails/games/{gameId}/language-codes/{languageCode}/images/{imageId}` — Deletes the game thumbnail.
- `discard-memory-store-queue-items` — `POST /cloud/v2/universes/{universe_id}/memory-store/queues/{queue_id}/items:discard` — Discard Memory Store Queue Items
- `edit-team-create-settings-for-a-universe` — `PATCH /legacy-develop/v1/universes/{universeId}/teamcreate` — Edit team create settings for a universe.
- `enable-or-disable-automatic-translation-for-a-game-and-language` — `PATCH /legacy-game-internationalization/v1/supported-languages/games/{gameId}/languages/{languageCode}/automatic-translation-status` — Enable or disable automatic translation for a game and language.
- `enable-or-disable-image-translation-for-a-game-and-language` — `PATCH /legacy-game-internationalization/v1/supported-languages/games/{gameId}/languages/{languageCode}/image-translation-status` — Enable or disable image translation for a game and language.
- `flush-memory-store` — `POST /cloud/v2/universes/{universe_id}/memory-store:flush` — Flush Memory Store
- `forecast-the-impact-of-restarting-game-servers-for-a-universe` — `GET /server-management/v1/universes/{universeId}/restarts:forecast` — Forecast the impact of restarting game servers for a universe.
- `generate-speech-asset` — `POST /cloud/v2/universes/{universe_id}:generateSpeechAsset` — Generate Speech Asset
- `generate-user-thumbnail` — `GET /cloud/v2/users/{user_id}:generateThumbnail` — Generate User Thumbnail
- `get-3d-object-for-an-outfit` — `GET /v1/users/outfit-3d` — Get 3d object for an outfit
- `get-all-icons-for-a-badge` — `GET /legacy-game-internationalization/v1/badges/{badgeId}/icons` — Get all icons for a badge
- `get-all-icons-for-a-developer-product` — `GET /legacy-game-internationalization/v1/developer-products/{developerProductId}/icons` — Get all icons for a developer product
- `get-all-icons-for-a-game` — `GET /legacy-game-internationalization/v1/game-icon/games/{gameId}` — Get all icons for a game
- `get-all-icons-for-a-game-pass` — `GET /legacy-game-internationalization/v1/game-passes/{gamePassId}/icons` — Get all icons for a game pass
- `get-all-names-and-descriptions-of-a-developer-product` — `GET /legacy-game-internationalization/v1/developer-products/{developerProductId}/name-description` — Get all names and descriptions of a developer product
- `get-all-names-and-descriptions-of-a-game-pass` — `GET /legacy-game-internationalization/v1/game-passes/{gamePassId}/name-description` — Get all names and descriptions of a game pass
- `get-asset-version` — `GET /assets/v1/assets/{assetId}/versions/{versionNumber}` — Get Asset Version
- `get-avatar-3d-object-for-a-user` — `GET /v1/users/avatar-3d` — Get Avatar 3d object for a user
- `get-creator-store-asset-details` — `GET /toolbox-service/v2/assets/{id}` — Get Creator Store Asset Details
- `get-creator-store-product` — `GET /cloud/v2/creator-store-products/{creator_store_product_id}` — Get Creator Store Product
- `get-data-store-entry` — `GET /cloud/v2/universes/{universe_id}/data-stores/{data_store_id}/entries/{entry_id}` — Get Data Store Entry
- `get-data-store-entry-by-scope-id-entries-by-entry-id` — `GET /cloud/v2/universes/{universe_id}/data-stores/{data_store_id}/scopes/{scope_id}/entries/{entry_id}` — Get Data Store Entry
- `get-developer-product-with-configuration-details` — `GET /developer-products/v2/universes/{universeId}/developer-products/{productId}/creator` — Get developer product with configuration details
- `get-entry` — `GET /datastores/v1/universes/{universeId}/standard-datastores/datastore/entries/entry` — Get entry.
- `get-entry-version` — `GET /datastores/v1/universes/{universeId}/standard-datastores/datastore/entries/entry/versions/version` — Get entry version.
- `get-game-pass-with-configuration-details` — `GET /game-passes/v1/universes/{universeId}/game-passes/{gamePassId}/creator` — Get game pass with configuration details
- `get-group` — `GET /cloud/v2/groups/{group_id}` — Get Group
- `get-group-role` — `GET /cloud/v2/groups/{group_id}/roles/{role_id}` — Get Group Role
- `get-instance` — `GET /cloud/v2/universes/{universe_id}/places/{place_id}/instances/{instance_id}` — Get Instance
- `get-legacy-game-internationalization-v1-badges-badgeid-name-description` — `GET /legacy-game-internationalization/v1/badges/{badgeId}/name-description` — get /legacy-game-internationalization/v1/badges/{badgeId}/name-description
- `get-limits-for-translation-table-entries-operations` — `GET /legacy-localization-tables/v1/localization-table/limits` — Get limits for translation table entries operations
- `get-luau-execution-session-task` — `GET /cloud/v2/universes/{universe_id}/places/{place_id}/versions/{version_id}/luau-execution-sessions/{luau_execution_session_id}/tasks/{task_id}` — Get Luau Execution Session Task
- `get-memory-store-flush-operation` — `GET /cloud/v2/universes/{universe_id}/memory-store/operations/{operation_id}` — Get Memory Store Flush Operation
- `get-memory-store-sorted-map-item` — `GET /cloud/v2/universes/{universe_id}/memory-store/sorted-maps/{sorted_map_id}/items/{item_id}` — Get Memory Store Sorted Map Item
- `get-ordered-data-store-entry` — `GET /cloud/v2/universes/{universe_id}/ordered-data-stores/{ordered_data_store_id}/scopes/{scope_id}/entries/{entry_id}` — Get Ordered Data Store Entry
- `get-place` — `GET /cloud/v2/universes/{universe_id}/places/{place_id}` — Get Place
- `get-public-key` — `GET /cloud/v2/universes/{universeId}/secrets/public-key` — Get Public Key
- `get-subscription` — `GET /cloud/v2/universes/{universe_id}/subscription-products/{subscription_product_id}/subscriptions/{subscription_id}` — Get Subscription
- `get-table-information-by-the-assetid-of-the-table` — `GET /legacy-localization-tables/v1/localization-table/tables/{assetId}` — Get table information by the assetId of the table.
- `get-table-information-by-the-id-of-the-table` — `GET /legacy-localization-tables/v1/localization-table/tables/{tableId}` — Get table information by the id of the table.
- `get-the-automatic-translation-status-of-supported-languages-for-a-game` — `GET /legacy-game-internationalization/v1/supported-languages/games/{gameId}/automatic-translation-status` — Get the automatic translation status of supported languages for a game.
- `get-the-result-of-an-asset-creation-or-update` — `GET /assets/v1/operations/{operationId}` — Get the result of an asset creation or update.
- `get-universe` — `GET /cloud/v2/universes/{universe_id}` — Get Universe
- `get-universedisplayinfo-automatic-translation-settings` — `GET /legacy-game-internationalization/v1/supported-languages/games/{gameId}/universe-display-info-automatic-translation-settings` — Get UniverseDisplayInfo automatic translation settings.
- `get-update-instance-operation` — `GET /cloud/v2/universes/{universe_id}/places/{place_id}/instances/{instance_id}/operations/{operation_id}` — Get Update Instance Operation
- `get-user` — `GET /cloud/v2/users/{user_id}` — Get User
- `get-user-restriction` — `GET /cloud/v2/universes/{universe_id}/places/{place_id}/user-restrictions/{user_restriction_id}` — Get User Restriction
- `get-user-restriction-by-universe-id-user-restrictions-by-user-restriction-id` — `GET /cloud/v2/universes/{universe_id}/user-restrictions/{user_restriction_id}` — Get User Restriction
- `get-user-thumbnail-generation-operation` — `GET /cloud/v2/users/{user_id}/operations/{operation_id}` — Get User Thumbnail Generation Operation
- `gets-a-batch-of-entries-for-a-table` — `GET /legacy-localization-tables/v1/localization-table/tables/{tableId}/entries` — Gets a batch of entries for a table.
- `gets-a-list-of-groups-that-a-user-can-manage` — `GET /legacy-develop/v1/user/groups/canmanage` — Gets a list of Groups that a user can manage.
- `gets-all-the-followings-between-a-user-with-userid-and-universes` — `GET /legacy-followings/v1/users/{userId}/universes` — Gets all the followings between a user with userId and universes
- `gets-all-universes-followed-by-a-user` — `GET /legacy-followings/v2/users/{userId}/universes` — Gets all universes followed by a user.
- `gets-available-filter-options-for-game-servers` — `GET /server-management/v1/universes/{universeId}/places/{placeId}/game-servers:filter-options` — Gets available filter options for game servers.
- `gets-full-config-with-metadata` — `GET /creator-configs-public-api/v1/configs/universes/{universeId}/repositories/{repository}/full` — Gets full config with metadata.
- `gets-group-policy-info-used-for-compliance` — `POST /legacy-groups/v1/groups/policies` — Gets group policy info used for compliance.
- `gets-groups-that-the-authenticated-user-has-requested-to-join` — `GET /legacy-groups/v1/user/groups/pending` — Gets groups that the authenticated user has requested to join
- `gets-published-config-values-without-metadata` — `GET /creator-configs-public-api/v1/configs/universes/{universeId}/repositories/{repository}` — Gets published config values without metadata.
- `gets-saves-from-a-collection` — `GET /toolbox-service/v1/saves` — Gets saves from a collection.
- `gets-teamcreate-settings-for-an-roblox-platform-universes-iuniverse` — `GET /legacy-develop/v1/universes/{universeId}/teamcreate` — Gets TeamCreate settings for an Roblox.Platform.Universes.IUniverse.
- `gets-teamcreate-settings-for-multiple-universes-specified-by-ids` — `GET /legacy-develop/v1/universes/multiget/teamcreate` — Gets TeamCreate settings for multiple universes specified by Ids
- `gets-the-current-draft` — `GET /creator-configs-public-api/v1/configs/universes/{universeId}/repositories/{repository}/draft` — Gets the current draft.
- `gets-the-group-s-audit-log` — `GET /legacy-groups/v1/groups/{groupId}/audit-log` — Gets the Group's audit log
- `gets-the-group-s-settings` — `GET /legacy-groups/v1/groups/{groupId}/settings` — Gets the Group's settings
- `gets-the-history-for-name-or-description-in-a-provided-language` — `POST /legacy-game-internationalization/v1/name-description/games/translation-history` — Gets the history for name or description in a provided language.
- `gets-the-number-of-entries-in-the-specified-table` — `GET /legacy-localization-tables/v1/localization-table/tables/{tableId}/entry-count` — Gets the number of entries in the specified table
- `gets-the-status-of-a-following-relationship-between-a-user-and-a-universe` — `GET /legacy-followings/v1/users/{userId}/universes/{universeId}/status` — Gets the status of a following relationship between a user and a universe.
- `gets-the-translation-history-for-each-entry-passed-in` — `POST /legacy-localization-tables/v1/localization-table/tables/{tableId}/entries/translation-history` — Gets the translation history for each entry passed in.
- `grant-a-subject-permission-to-multiple-assets-authorization-is-required-to-grant-permissions-to-the-subject-and-asset-ids-in-the-request` — `PATCH /asset-permissions-api/v1/assets/permissions` — Grant a subject permission to multiple assets.
            
Authorization is required to grant permissions to the subject and asset IDs in the request.
- `increment-data-store-entry` — `POST /cloud/v2/universes/{universe_id}/data-stores/{data_store_id}/entries/{entry_id}:increment` — Increment Data Store Entry
- `increment-data-store-entry-by-scope-id-entries-by-entry-id-increment` — `POST /cloud/v2/universes/{universe_id}/data-stores/{data_store_id}/scopes/{scope_id}/entries/{entry_id}:increment` — Increment Data Store Entry
- `increment-entry` — `POST /datastores/v1/universes/{universeId}/standard-datastores/datastore/entries/entry/increment` — Increment entry
- `increment-ordered-data-store-entry` — `POST /cloud/v2/universes/{universe_id}/ordered-data-stores/{ordered_data_store_id}/scopes/{scope_id}/entries/{entry_id}:increment` — Increment Ordered Data Store Entry
- `launch-a-game-server-restart-for-a-universe` — `POST /server-management/v1/universes/{universeId}/restarts` — Launch a game server restart for a universe.
- `list-asset-quotas` — `GET /cloud/v2/users/{user_id}/asset-quotas` — List Asset Quotas
- `list-asset-versions-of-an-asset` — `GET /assets/v1/assets/{assetId}/versions` — List Asset Versions of an Asset
- `list-data-store-entries` — `GET /cloud/v2/universes/{universe_id}/data-stores/{data_store_id}/entries` — List Data Store Entries
- `list-data-store-entries-scopes-by-scope-id-entries` — `GET /cloud/v2/universes/{universe_id}/data-stores/{data_store_id}/scopes/{scope_id}/entries` — List Data Store Entries
- `list-data-store-entry-revisions` — `GET /cloud/v2/universes/{universe_id}/data-stores/{data_store_id}/entries/{entry_id}:listRevisions` — List Data Store Entry Revisions
- `list-data-store-entry-revisions-by-scope-id-entries-by-entry-id-listrevisions` — `GET /cloud/v2/universes/{universe_id}/data-stores/{data_store_id}/scopes/{scope_id}/entries/{entry_id}:listRevisions` — List Data Store Entry Revisions
- `list-data-stores` — `GET /cloud/v2/universes/{universe_id}/data-stores` — List Data Stores
- `list-data-stores-in-an-experience` — `GET /datastores/v1/universes/{universeId}/standard-datastores` — List data stores in an experience
- `list-developer-products-by-universe-with-configuration-details` — `GET /developer-products/v2/universes/{universeId}/developer-products/creator` — List developer products by universe with configuration details
- `list-entries` — `GET /datastores/v1/universes/{universeId}/standard-datastores/datastore/entries` — List entries
- `list-entry-versions` — `GET /datastores/v1/universes/{universeId}/standard-datastores/datastore/entries/entry/versions` — List entry versions
- `list-game-passes-by-universe-with-configuration-details` — `GET /game-passes/v1/universes/{universeId}/game-passes/creator` — List game passes by universe with configuration details
- `list-group-forum-categories` — `GET /cloud/v2/groups/{group_id}/forum-categories` — List Group Forum Categories
- `list-group-forum-comments` — `GET /cloud/v2/groups/{group_id}/forum-categories/{forum_category_id}/posts/{post_id}/comments` — List Group Forum Comments
- `list-group-forum-posts` — `GET /cloud/v2/groups/{group_id}/forum-categories/{forum_category_id}/posts` — List Group Forum Posts
- `list-group-join-requests` — `GET /cloud/v2/groups/{group_id}/join-requests` — List Group Join Requests
- `list-group-memberships` — `GET /cloud/v2/groups/{group_id}/memberships` — List Group Memberships
- `list-group-roles` — `GET /cloud/v2/groups/{group_id}/roles` — List Group Roles
- `list-instance-children` — `GET /cloud/v2/universes/{universe_id}/places/{place_id}/instances/{instance_id}:listChildren` — List Instance Children
- `list-inventory-items` — `GET /cloud/v2/users/{user_id}/inventory-items` — List Inventory Items
- `list-luau-execution-session-task-logs` — `GET /cloud/v2/universes/{universe_id}/places/{place_id}/versions/{version_id}/luau-execution-sessions/{luau_execution_session_id}/tasks/{task_id}/logs` — List Luau Execution Session Task Logs
- `list-memory-store-sorted-map-items` — `GET /cloud/v2/universes/{universe_id}/memory-store/sorted-maps/{sorted_map_id}/items` — List Memory Store Sorted Map Items
- `list-of-users-in-the-active-team-create-session` — `GET /legacy-develop/v1/places/{placeId}/teamcreate/active_session/members` — List of users in the active Team Create session
- `list-ordered-data-store-entries` — `GET /cloud/v2/universes/{universe_id}/ordered-data-stores/{ordered_data_store_id}/scopes/{scope_id}/entries` — List Ordered Data Store Entries
- `list-restart-statuses-for-a-universe` — `GET /server-management/v1/universes/{universeId}/restarts` — List restart statuses for a universe.
- `list-secrets` — `GET /cloud/v2/universes/{universeId}/secrets` — List Secrets
- `list-user-restriction-logs` — `GET /cloud/v2/universes/{universe_id}/user-restrictions:listLogs` — List User Restriction Logs
- `list-user-restrictions` — `GET /cloud/v2/universes/{universe_id}/places/{place_id}/user-restrictions` — List User Restrictions
- `list-user-restrictions-universes-by-universe-id-user-restrictions` — `GET /cloud/v2/universes/{universe_id}/user-restrictions` — List User Restrictions
- `lists-game-servers-for-a-specific-place-version` — `GET /server-management/v1/universes/{universeId}/places/{placeId}/versions/{versionNumber}/game-servers` — Lists game servers for a specific place version.
- `lists-revision-history` — `GET /creator-configs-public-api/v1/configs/universes/{universeId}/repositories/{repository}/revisions` — Lists revision history.
- `metadata-for-autolocalization-configuration` — `GET /legacy-localization-tables/v1/autolocalization/metadata` — Metadata for AutoLocalization Configuration
- `ordereddatastores-createentry` — `POST /ordered-data-stores/v1/universes/{universeId}/orderedDataStores/{orderedDataStore}/scopes/{scope}/entries` — OrderedDataStores_CreateEntry
- `ordereddatastores-deleteentry` — `DELETE /ordered-data-stores/v1/universes/{universeId}/orderedDataStores/{orderedDataStore}/scopes/{scope}/entries/{entry}` — OrderedDataStores_DeleteEntry
- `ordereddatastores-getentry` — `GET /ordered-data-stores/v1/universes/{universeId}/orderedDataStores/{orderedDataStore}/scopes/{scope}/entries/{entry}` — OrderedDataStores_GetEntry
- `ordereddatastores-incremententry` — `POST /ordered-data-stores/v1/universes/{universeId}/orderedDataStores/{orderedDataStore}/scopes/{scope}/entries/{entry}:increment` — OrderedDataStores_IncrementEntry
- `ordereddatastores-listentries` — `GET /ordered-data-stores/v1/universes/{universeId}/orderedDataStores/{orderedDataStore}/scopes/{scope}/entries` — OrderedDataStores_ListEntries
- `ordereddatastores-updateentry` — `PATCH /ordered-data-stores/v1/universes/{universeId}/orderedDataStores/{orderedDataStore}/scopes/{scope}/entries/{entry}` — OrderedDataStores_UpdateEntry
- `orders-the-specified-image-ids-for-the-game-thumbnails` — `POST /legacy-game-internationalization/v1/game-thumbnails/games/{gameId}/language-codes/{languageCode}/images/order` — Orders the specified image Ids for the game thumbnails.
- `overwrites-a-badge-icon-with-a-new-one` — `POST /legacy-publish/v1/badges/{badgeId}/icon` — Overwrites a badge icon with a new one.
- `overwrites-the-entire-draft-with-the-request-payload` — `PUT /creator-configs-public-api/v1/configs/universes/{universeId}/repositories/{repository}/draft:overwrite` — Overwrites the entire draft with the request payload.
- `partially-updates-the-draft` — `PATCH /creator-configs-public-api/v1/configs/universes/{universeId}/repositories/{repository}/draft` — Partially updates the draft.
- `places-createplaceversionapikey` — `POST /universes/v1/{universeId}/places/{placeId}/versions` — Places_CreatePlaceVersionApiKey
- `post-legacy-localization-tables-v1-autolocalization-games-gameid-autolocalizationtable` — `POST /legacy-localization-tables/v1/autolocalization/games/{gameId}/autolocalizationtable` — post /legacy-localization-tables/v1/autolocalization/games/{gameId}/autolocalizationtable
- `publish-a-cross-server-message-to-a-universe` — `POST /messaging-service/v1/universes/{universeId}/topics/{topic}` — Publish a cross-server message to a universe
- `publish-universe-message` — `POST /cloud/v2/universes/{universe_id}:publishMessage` — Publish Universe Message
- `publishes-draft-changes` — `POST /creator-configs-public-api/v1/configs/universes/{universeId}/repositories/{repository}/publish` — Publishes draft changes.
- `read-memory-store-queue-items` — `GET /cloud/v2/universes/{universe_id}/memory-store/queues/{queue_id}/items:read` — Read Memory Store Queue Items
- `removes-a-user-from-a-teamcreate-permissions-list` — `DELETE /legacy-develop/v1/universes/{universeId}/teamcreate/memberships` — Removes a user from a TeamCreate permissions list.
- `resets-deletes-the-current-draft` — `DELETE /creator-configs-public-api/v1/configs/universes/{universeId}/repositories/{repository}/draft` — Resets (deletes) the current draft.
- `restart-universe-servers` — `POST /cloud/v2/universes/{universe_id}:restartServers` — Restart Universe Servers
- `restores-an-archived-asset` — `POST /assets/v1/assets/{assetId}:restore` — Restores an archived asset.
- `retrieve-specific-asset-metadata-include-the-readmask-parameter-for-additional-asset-metadata` — `GET /assets/v1/assets/{assetId}` — Retrieve specific asset metadata. Include the `readMask` parameter for additional asset metadata.
- `retrieves-an-asset-by-its-id-and-version-number-with-opencloud-auth` — `GET /asset-delivery-api/v1/assetId/{assetId}/version/{versionNumber}` — Retrieves an asset by its ID and version number with OpenCloud auth.
- `retrieves-an-asset-by-its-id-with-opencloud-auth` — `GET /asset-delivery-api/v1/assetId/{assetId}` — Retrieves an asset by its ID with OpenCloud auth.
- `returns-an-array-of-granted-and-declined-permissions-related-to-the-universes-with-the-ids-in-ids-for-the-authenticated-user` — `GET /legacy-develop/v1/universes/multiget/permissions` — Returns an array of granted and declined permissions related to the universes with the ids in ids for the authenticated user.
- `returns-list-of-granted-and-declined-permissions-related-to-the-universe-with-the-id-universeid-for-authenticated-user` — `GET /legacy-develop/v1/universes/{universeId}/permissions` — Returns list of granted and declined permissions related to the universe with the id universeId for authenticated user
- `rollback-an-asset-to-a-previous-version` — `POST /assets/v1/assets/{assetId}/versions:rollback` — Rollback an asset to a previous version.
- `search-creator-store-assets` — `GET /toolbox-service/v2/assets:search` — Search Creator Store Assets
- `search-creator-store-assets-toolbox-service-v2-assets-search` — `POST /toolbox-service/v2/assets:search` — Search Creator Store Assets
- `set-entry` — `POST /datastores/v1/universes/{universeId}/standard-datastores/datastore/entries/entry` — Set entry.
- `sets-a-game-s-auto-localization-related-settings` — `PATCH /legacy-localization-tables/v1/autolocalization/games/{gameId}/settings` — Sets a game's auto-localization related settings
- `sets-group-status` — `PATCH /legacy-groups/v1/groups/{groupId}/status` — Sets group status
- `sets-the-source-language-of-a-game` — `PATCH /legacy-game-internationalization/v1/source-language/games/{gameId}` — Sets the source language of a game
- `snapshot-data-stores` — `POST /cloud/v2/universes/{universe_id}/data-stores:snapshot` — Snapshot Data Stores
- `stages-a-revert-to-a-previous-revision` — `POST /creator-configs-public-api/v1/configs/universes/{universeId}/repositories/{repository}/revisions/{revisionId}/restore` — Stages a revert to a previous revision.
- `thumbnails-assets` — `GET /v1/assets-thumbnail-3d` — Thumbnails assets.
- `translate-text` — `POST /cloud/v2/universes/{universe_id}:translateText` — Translate Text
- `unassign-role-group-membership` — `POST /cloud/v2/groups/{group_id}/memberships/{membership_id}:unassignRole` — Unassign Role Group Membership
- `undelete-data-store` — `POST /cloud/v2/universes/{universe_id}/data-stores/{data_store_id}:undelete` — Undelete Data Store
- `update-a-badge-s-icon` — `POST /legacy-game-internationalization/v1/badges/{badgeId}/icons/language-codes/{languageCode}` — Update a badge's icon
- `update-a-developer-product-s-icon` — `POST /legacy-game-internationalization/v1/developer-products/{developerProductId}/icons/language-codes/{languageCode}` — Update a developer product's icon
- `update-a-game-pass-s-icon` — `POST /legacy-game-internationalization/v1/game-passes/{gamePassId}/icons/language-codes/{languageCode}` — Update a game pass's icon
- `update-a-game-s-icon` — `POST /legacy-game-internationalization/v1/game-icon/games/{gameId}/language-codes/{languageCode}` — Update a game's icon
- `update-creator-store-product` — `PATCH /cloud/v2/creator-store-products/{creator_store_product_id}` — Update Creator Store Product
- `update-data-store-entry` — `PATCH /cloud/v2/universes/{universe_id}/data-stores/{data_store_id}/entries/{entry_id}` — Update Data Store Entry
- `update-data-store-entry-by-scope-id-entries-by-entry-id` — `PATCH /cloud/v2/universes/{universe_id}/data-stores/{data_store_id}/scopes/{scope_id}/entries/{entry_id}` — Update Data Store Entry
- `update-developer-product` — `PATCH /developer-products/v2/universes/{universeId}/developer-products/{productId}` — Update developer product
- `update-game-pass` — `PATCH /game-passes/v1/universes/{universeId}/game-passes/{gamePassId}` — Update game pass
- `update-group-membership` — `PATCH /cloud/v2/groups/{group_id}/memberships/{membership_id}` — Update Group Membership
- `update-instance` — `PATCH /cloud/v2/universes/{universe_id}/places/{place_id}/instances/{instance_id}` — Update Instance
- `update-localized-description-of-a-badge` — `PATCH /legacy-game-internationalization/v1/badges/{badgeId}/description/language-codes/{languageCode}` — Update localized description of a badge
- `update-localized-description-of-a-developer-product` — `PATCH /legacy-game-internationalization/v1/developer-products/{developerProductId}/description/language-codes/{languageCode}` — Update localized description of a developer product
- `update-localized-description-of-a-game-pass` — `PATCH /legacy-game-internationalization/v1/game-passes/{gamePassId}/description/language-codes/{languageCode}` — Update localized description of a game pass
- `update-localized-name-and-description-of-a-badge` — `PATCH /legacy-game-internationalization/v1/badges/{badgeId}/name-description/language-codes/{languageCode}` — Update localized name and description of a badge
- `update-localized-name-and-description-of-a-developer-product` — `PATCH /legacy-game-internationalization/v1/developer-products/{developerProductId}/name-description/language-codes/{languageCode}` — Update localized name and description of a developer product
- `update-localized-name-and-description-of-a-game-pass` — `PATCH /legacy-game-internationalization/v1/game-passes/{gamePassId}/name-description/language-codes/{languageCode}` — Update localized name and description of a game pass
- `update-localized-name-of-a-badge` — `PATCH /legacy-game-internationalization/v1/badges/{badgeId}/name/language-codes/{languageCode}` — Update localized name of a badge
- `update-localized-name-of-a-developer-product` — `PATCH /legacy-game-internationalization/v1/developer-products/{developerProductId}/name/language-codes/{languageCode}` — Update localized name of a developer product
- `update-localized-name-of-a-game-pass` — `PATCH /legacy-game-internationalization/v1/game-passes/{gamePassId}/name/language-codes/{languageCode}` — Update localized name of a game pass
- `update-memory-store-sorted-map-item` — `PATCH /cloud/v2/universes/{universe_id}/memory-store/sorted-maps/{sorted_map_id}/items/{item_id}` — Update Memory Store Sorted Map Item
- `update-ordered-data-store-entry` — `PATCH /cloud/v2/universes/{universe_id}/ordered-data-stores/{ordered_data_store_id}/scopes/{scope_id}/entries/{entry_id}` — Update Ordered Data Store Entry
- `update-place` — `PATCH /cloud/v2/universes/{universe_id}/places/{place_id}` — Update Place
- `update-secret` — `PATCH /cloud/v2/universes/{universeId}/secrets/{secretId}` — Update Secret
- `update-the-switch-which-controls-if-the-universedisplayinformation-should-be-automatically-translated` — `PATCH /legacy-game-internationalization/v1/supported-languages/games/{gameId}/languages/{languageCode}/universe-display-info-automatic-translation-settings` — Update the switch which controls if the UniverseDisplayInformation should be automatically translated.
- `update-universe` — `PATCH /cloud/v2/universes/{universe_id}` — Update Universe
- `update-user-restriction` — `PATCH /cloud/v2/universes/{universe_id}/places/{place_id}/user-restrictions/{user_restriction_id}` — Update User Restriction
- `update-user-restriction-by-universe-id-user-restrictions-by-user-restriction-id` — `PATCH /cloud/v2/universes/{universe_id}/user-restrictions/{user_restriction_id}` — Update User Restriction
- `updates-a-game-s-name-and-or-description-in-multiple-languages` — `PATCH /legacy-game-internationalization/v1/name-description/games/{gameId}` — Updates a game's name and/or description in multiple languages.
- `updates-an-asset-with-provided-content-and-metadata` — `PATCH /assets/v1/assets/{assetId}` — Updates an asset with provided content and metadata.
- `updates-badge-configuration` — `PATCH /legacy-badges/v1/badges/{badgeId}` — Updates badge configuration.
- `updates-the-game-thumbnail-alt-text` — `POST /legacy-game-internationalization/v1/game-thumbnails/games/{gameId}/language-codes/{languageCode}/alt-text` — Updates the game thumbnail alt text.
- `updates-the-group-s-settings` — `PATCH /legacy-groups/v1/groups/{groupId}/notification-preference` — Updates the group's settings
- `updates-the-group-s-settings-groups-by-groupid-settings` — `PATCH /legacy-groups/v1/groups/{groupId}/settings` — Updates the group's settings
- `updates-the-groups-description` — `PATCH /legacy-groups/v1/groups/{groupId}/description` — Updates the groups description
- `updates-the-tables-contents-based-on-what-is-provided` — `PATCH /legacy-localization-tables/v1/localization-table/tables/{tableId}` — Updates the tables contents based on what is provided.
- `uploads-the-game-thumbnail` — `POST /legacy-game-internationalization/v1/game-thumbnails/games/{gameId}/language-codes/{languageCode}/image` — Uploads the game thumbnail.
