mod support;

use std::time::Duration;

use rbxcloud::rbx::{
    avatar_bundle_context::{AvatarBundleContextClient, CreateBundleContextInput},
    client::{AsyncClient, ClientPolicy, Server},
    collectible_management::{
        BulkUpdateCollectiblesInput, CollectibleClient, PublishCollectibleInput,
        UpdateCollectibleInput,
    },
    credentials::{Credentials, SecretString},
    item_folders::ItemFolderClient,
};
use support::{Reply, TestServer};

#[tokio::test]
async fn bundle_folder_and_collectible_mutations_use_item_configuration_host() {
    let default = TestServer::start((0..6).map(|_| Reply::json(200, "{}")).collect());
    let item_configuration = TestServer::start((0..6).map(|_| Reply::json(200, "{}")).collect());
    let policy = ClientPolicy::loopback(default.addr())
        .unwrap()
        .with_loopback_server(Server::ItemConfiguration, item_configuration.addr())
        .unwrap()
        .timeout(Duration::from_secs(2))
        .retry_delay(Duration::from_millis(1));
    let transport = AsyncClient::new(
        Credentials::api_key(SecretString::new("item-configuration-host-test-key")),
        policy,
    )
    .await
    .unwrap();

    AvatarBundleContextClient::new(transport.clone())
        .create_context(CreateBundleContextInput::default())
        .await
        .unwrap();
    let folders = ItemFolderClient::new(transport.clone());
    folders.add_items("folder".into(), vec![]).await.unwrap();
    folders.delete_items("folder".into(), vec![]).await.unwrap();
    let collectibles = CollectibleClient::new(transport);
    collectibles
        .publish(PublishCollectibleInput::default())
        .await
        .unwrap();
    collectibles
        .bulk_update(BulkUpdateCollectiblesInput::default())
        .await
        .unwrap();
    collectibles
        .update("item", UpdateCollectibleInput::default())
        .await
        .unwrap();

    assert!(
        default.requests().is_empty(),
        "item-configuration operations must not use apis.roblox.com"
    );
    let targets: Vec<String> = item_configuration
        .requests()
        .into_iter()
        .map(|request| request.target)
        .collect();
    assert_eq!(
        targets,
        [
            "/v1/bundles/create-context",
            "/v1/folders/folder/items",
            "/v1/folders/folder/items",
            "/v1/collectibles",
            "/v1/collectibles/bulk-update",
            "/v1/collectibles/item",
        ]
    );
}
