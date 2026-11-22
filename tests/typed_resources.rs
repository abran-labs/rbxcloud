mod support;
use rbxcloud::rbx::{
    client::{AsyncClient, ClientPolicy, Server},
    credentials::{Credentials, SecretString},
    generated::resources::*,
};
use std::time::Duration;
use support::{Reply, TestServer};

async fn client(server: &TestServer, thumbnails: Option<&TestServer>) -> ResourcesCloudClient {
    let mut policy = ClientPolicy::loopback(server.addr())
        .unwrap()
        .timeout(Duration::from_secs(2))
        .retry_delay(Duration::from_millis(1));
    if let Some(server) = thumbnails {
        policy = policy
            .with_loopback_server(Server::Thumbnails, server.addr())
            .unwrap();
    }
    ResourcesCloudClient::new(
        AsyncClient::new(
            Credentials::api_key(SecretString::new("resources-contract-test-key")),
            policy,
        )
        .await
        .unwrap(),
    )
}

#[tokio::test]
async fn resource_paths_queries_and_thumbnail_host_are_fixed() {
    let server = TestServer::start(
        (0..4)
            .map(|_| Reply::json(400, r#"{"message":"contract stop"}"#))
            .collect(),
    );
    let thumbnails = TestServer::start(vec![Reply::json(400, r#"{"message":"contract stop"}"#)]);
    let client = client(&server, Some(&thumbnails)).await;
    let _ = client
        .game_localization_list_badge_icons(GameLocalizationListBadgeIconsParams {
            badge_id: 9,
            width: Some(100),
            height: Some(50),
        })
        .await;
    let _ = client
        .service_efficiency_get_account_for_user(ServiceEfficiencyGetAccountForUserParams {
            user_id: 5,
        })
        .await;
    let _ = client
        .item_folders_list_items(ItemFoldersListItemsParams {
            folder_id: "f".into(),
            limit: Some(10),
            cursor: Some("c".into()),
        })
        .await;
    let _ = client
        .secrets_list(SecretsListParams {
            universe_id: 1,
            limit: Some(2),
            cursor: None,
        })
        .await;
    let _ = client
        .thumbnails_get_asset_3d(ThumbnailsGetAsset3dParams {
            asset_id: 8,
            use_gltf: Some(true),
            roblox_place_id: Some(3),
        })
        .await;
    let requests = server.requests();
    assert_eq!(
        requests[0].target,
        "/legacy-game-internationalization/v1/badges/9/icons?width=100&height=50"
    );
    assert_eq!(
        requests[1].target,
        "/service-efficiency-api/v1/accounts/users/5"
    );
    assert_eq!(requests[2].target, "/v1/folders/f/items?limit=10&cursor=c");
    assert_eq!(requests[3].target, "/cloud/v2/universes/1/secrets?limit=2");
    let thumbnail = thumbnails.requests();
    assert_eq!(thumbnail[0].target, "/v1/assets-thumbnail-3d?assetId=8&useGltf=true");
    assert_eq!(thumbnail[0].header("roblox-place-id"), Some("3"));
}

#[test]
fn resource_cli_exposes_expected_groups_and_no_asset_delivery_placeholder() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_rbxcloud"))
        .args(["resource-tools", "--help"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let help = String::from_utf8(output.stdout).unwrap();
    for group in [
        "collectibles",
        "creator-store",
        "game-localization",
        "item-folders",
        "marketplace-items",
        "secrets",
        "service-efficiency",
        "thumbnails",
    ] {
        assert!(help.contains(group));
    }
    assert!(!help.contains("asset-delivery"));
}
