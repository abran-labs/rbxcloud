mod support;

use rbxcloud::rbx::{
    asset_permission_grants::*,
    client::{AsyncClient, ClientPolicy},
    credentials::{Credentials, SecretString},
    economy_uploads::*,
};
use std::{
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};
use support::{parse_multipart, Reply, TestServer};

async fn transport(server: &TestServer) -> AsyncClient {
    let policy = ClientPolicy::loopback(server.addr())
        .unwrap()
        .timeout(Duration::from_secs(2))
        .retry_delay(Duration::from_millis(1));
    AsyncClient::new(
        Credentials::api_key(SecretString::new("economy-upload-test-key")),
        policy,
    )
    .await
    .unwrap()
}
fn image_file() -> PathBuf {
    static NEXT_FILE: AtomicU64 = AtomicU64::new(0);
    let sequence = NEXT_FILE.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "rbxcloud-economy-{}-{sequence}.png",
        std::process::id()
    ));
    std::fs::write(&path, b"economy-image").unwrap();
    path
}
fn text(parts: &[support::PartSeen], name: &str) -> String {
    String::from_utf8(
        parts
            .iter()
            .find(|part| part.name == name)
            .unwrap()
            .data
            .clone(),
    )
    .unwrap()
}

#[tokio::test]
async fn developer_product_create_and_update_send_typed_multipart_fields() {
    let server = TestServer::start(vec![Reply::json(200, "{}"), Reply::json(200, "{}")]);
    let client = EconomyUploadClient::new(transport(&server).await);
    let path = image_file();
    client
        .create_developer_product(
            7,
            CreateDeveloperProductInput {
                name: "Coins".into(),
                description: Some("Currency".into()),
                is_for_sale: Some(true),
                price: Some(100),
                is_regional_pricing_enabled: Some(false),
                is_managed_pricing_enabled: Some(true),
            },
            Some(&path),
            "image/png",
        )
        .await
        .unwrap();
    client
        .update_developer_product(
            7,
            123,
            UpdateDeveloperProductInput {
                name: Some("More coins".into()),
                description: None,
                is_for_sale: Some(false),
                price: Some(200),
                is_regional_pricing_enabled: None,
                store_page_enabled: Some(true),
                is_managed_pricing_enabled: Some(false),
            },
            None,
            "image/png",
        )
        .await
        .unwrap();
    std::fs::remove_file(path).unwrap();

    let requests = server.requests();
    assert_eq!(
        requests[0].target,
        "/developer-products/v2/universes/7/developer-products"
    );
    let create = parse_multipart(
        requests[0].header("content-type").unwrap(),
        &requests[0].body,
    );
    assert_eq!(text(&create, "name"), "Coins");
    assert_eq!(text(&create, "price"), "100");
    assert_eq!(text(&create, "isManagedPricingEnabled"), "true");
    assert_eq!(
        create
            .iter()
            .find(|part| part.name == "imageFile")
            .unwrap()
            .data,
        b"economy-image"
    );
    assert_eq!(
        requests[1].target,
        "/developer-products/v2/universes/7/developer-products/123"
    );
    let update = parse_multipart(
        requests[1].header("content-type").unwrap(),
        &requests[1].body,
    );
    assert_eq!(text(&update, "storePageEnabled"), "true");
    assert!(!update.iter().any(|part| part.name == "imageFile"));
}

#[tokio::test]
async fn game_pass_create_and_update_support_image_file_and_alias() {
    let server = TestServer::start(vec![Reply::json(200, "{}"), Reply::json(200, "{}")]);
    let client = EconomyUploadClient::new(transport(&server).await);
    let path = image_file();
    client
        .create_game_pass(
            7,
            CreateGamePassInput {
                name: "VIP".into(),
                description: Some("Benefits".into()),
                is_for_sale: Some(true),
                price: Some(50),
                is_regional_pricing_enabled: None,
                is_managed_pricing_enabled: Some(true),
                is_trial_enabled: Some(true),
            },
            Some(&path),
            "image/png",
        )
        .await
        .unwrap();
    client
        .update_game_pass(
            7,
            123,
            UpdateGamePassInput {
                name: Some("VIP+".into()),
                description: None,
                is_for_sale: None,
                price: None,
                is_regional_pricing_enabled: None,
                is_managed_pricing_enabled: None,
                is_trial_enabled: Some(false),
            },
            None,
            Some(&path),
            "image/png",
        )
        .await
        .unwrap();
    std::fs::remove_file(path).unwrap();

    let requests = server.requests();
    assert_eq!(
        requests[0].target,
        "/game-passes/v1/universes/7/game-passes"
    );
    let create = parse_multipart(
        requests[0].header("content-type").unwrap(),
        &requests[0].body,
    );
    assert_eq!(text(&create, "isTrialEnabled"), "true");
    assert!(create.iter().any(|part| part.name == "imageFile"));
    assert_eq!(
        requests[1].target,
        "/game-passes/v1/universes/7/game-passes/123"
    );
    let update = parse_multipart(
        requests[1].header("content-type").unwrap(),
        &requests[1].body,
    );
    assert!(update.iter().any(|part| part.name == "file"));
    assert!(!update.iter().any(|part| part.name == "imageFile"));
    assert_eq!(text(&update, "isTrialEnabled"), "false");
}

#[tokio::test]
async fn badge_create_and_icon_update_use_distinct_file_field_names() {
    let server = TestServer::start(vec![Reply::json(200, "{}"), Reply::json(200, "{}")]);
    let client = EconomyUploadClient::new(transport(&server).await);
    let path = image_file();
    client
        .create_badge(
            7,
            CreateBadgeInput {
                name: Some("Explorer".into()),
                description: Some("Explore".into()),
                payment_source_type: Some(BadgePaymentSource::Group),
                expected_cost: Some(100),
                is_active: Some(true),
            },
            Some(&path),
            "image/png",
        )
        .await
        .unwrap();
    client
        .update_badge_icon(123, &path, "image/png")
        .await
        .unwrap();
    std::fs::remove_file(path).unwrap();

    let requests = server.requests();
    assert_eq!(requests[0].target, "/legacy-badges/v1/universes/7/badges");
    let create = parse_multipart(
        requests[0].header("content-type").unwrap(),
        &requests[0].body,
    );
    assert_eq!(text(&create, "paymentSourceType"), "2");
    assert!(create.iter().any(|part| part.name == "files"));
    assert_eq!(requests[1].target, "/legacy-publish/v1/badges/123/icon");
    let update = parse_multipart(
        requests[1].header("content-type").unwrap(),
        &requests[1].body,
    );
    assert!(update.iter().any(|part| part.name == "Files"));
}

#[tokio::test]
async fn asset_permission_batch_grant_serializes_typed_requests() {
    let server = TestServer::start(vec![Reply::json(200, "{}")]);
    let client = AssetPermissionClient::new(transport(&server).await);
    client
        .batch_grant(BatchGrantPermissionsInput {
            subject_type: Some(AssetSubjectType::Group),
            subject_id: Some("42".into()),
            action: Some(AssetGrantAction::Use),
            requests: Some(vec![AssetGrantInput {
                asset_id: Some(123),
                grant_to_dependencies: Some(true),
                parent_version_number: Some(4),
            }]),
            asset_ids: Some(vec![999]),
            enable_deep_access_check: Some(false),
        })
        .await
        .unwrap();

    let requests = server.requests();
    assert_eq!(
        requests[0].target,
        "/asset-permissions-api/v1/assets/permissions"
    );
    let body: serde_json::Value = serde_json::from_str(&requests[0].body_text()).unwrap();
    assert_eq!(body["subjectType"], "Group");
    assert_eq!(body["action"], "Use");
    assert_eq!(body["requests"][0]["assetId"], 123);
    assert_eq!(body["requests"][0]["grantToDependencies"], true);
    assert_eq!(body["assetIds"], serde_json::json!([999]));
}
