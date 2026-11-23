mod support;

use rbxcloud::rbx::{
    avatar_bundle_context::*,
    client::{AsyncClient, ClientPolicy, Server},
    creator_store_management::*,
    credentials::{Credentials, SecretString},
    item_folders::*,
};
use std::{collections::BTreeMap, time::Duration};
use support::{Reply, TestServer};

async fn transport(server: &TestServer) -> AsyncClient {
    let policy = ClientPolicy::loopback(server.addr())
        .unwrap()
        .with_loopback_server(Server::ItemConfiguration, server.addr())
        .unwrap()
        .timeout(Duration::from_secs(2))
        .retry_delay(Duration::from_millis(1));
    AsyncClient::new(
        Credentials::api_key(SecretString::new("catalog-management-test-key")),
        policy,
    )
    .await
    .unwrap()
}

#[tokio::test]
async fn creator_store_products_use_typed_money_and_immutable_create_fields() {
    let server = TestServer::start(vec![Reply::json(200, "{}"), Reply::json(200, "{}")]);
    let client = CreatorStoreProductClient::new(transport(&server).await);
    client
        .create_product(CreateCreatorStoreProductInput {
            base_price: Some(MoneyInput {
                currency_code: Some("USD".into()),
                quantity: Some(DecimalInput {
                    significand: Some(4_990_000_000),
                    exponent: Some(-9),
                }),
            }),
            published: Some(true),
            plugin_asset_id: Some("123".into()),
            ..Default::default()
        })
        .await
        .unwrap();
    client
        .update_product(
            "456".into(),
            UpdateCreatorStoreProductInput {
                base_price: Some(MoneyInput {
                    currency_code: Some("USD".into()),
                    quantity: Some(DecimalInput {
                        significand: Some(0),
                        exponent: Some(0),
                    }),
                }),
                published: Some(false),
                ..Default::default()
            },
        )
        .await
        .unwrap();

    let requests = server.requests();
    assert_eq!(requests[0].target, "/cloud/v2/creator-store-products");
    let create: serde_json::Value = serde_json::from_str(&requests[0].body_text()).unwrap();
    assert_eq!(
        create["basePrice"]["quantity"]["significand"],
        4_990_000_000i64
    );
    assert_eq!(create["basePrice"]["quantity"]["exponent"], -9);
    assert_eq!(create["pluginAssetId"], "123");
    assert_eq!(requests[1].target, "/cloud/v2/creator-store-products/456");
    assert_eq!(
        requests[1].body_text(),
        r#"{"basePrice":{"currencyCode":"USD","quantity":{"significand":0,"exponent":0}},"published":false}"#
    );
}

#[tokio::test]
async fn toolbox_bulk_delete_and_search_use_typed_targets_and_filters() {
    let server = TestServer::start(vec![Reply::json(200, "{}"), Reply::json(200, "{}")]);
    let client = ToolboxClient::new(transport(&server).await);
    client
        .bulk_delete_saves(vec![DeleteSaveInput {
            target_type: SearchCategoryType::Plugin,
            target_id: 123,
            collection_name: None,
        }])
        .await
        .unwrap();
    client
        .search_assets(SearchCreatorStoreAssetsInput {
            search_category_type: Some(SearchCategoryType::Audio),
            query: Some("ambient".into()),
            max_page_size: Some(25),
            sort_direction: Some(CreatorStoreSortDirection::Descending),
            sort_category: Some(CreatorStoreSortCategory::Top),
            facets: Some(vec!["calm".into()]),
            audio_search_filters: Some(AudioSearchFiltersInput {
                audio_types: Some(vec![SearchAudioType::Music]),
                min_duration_seconds: Some(10),
                max_duration_seconds: Some(120),
                artist: Some("Composer".into()),
                album: None,
                music_chart_type: Some(MusicChartType::Month),
            }),
            model_search_filters: None,
            ..Default::default()
        })
        .await
        .unwrap();

    let requests = server.requests();
    assert_eq!(requests[0].target, "/toolbox-service/v1/saves:bulkDelete");
    assert_eq!(
        requests[0].body_text(),
        r#"{"targets":[{"targetType":"Plugin","targetId":123}]}"#
    );
    assert_eq!(requests[1].target, "/toolbox-service/v2/assets:search");
    let search: serde_json::Value = serde_json::from_str(&requests[1].body_text()).unwrap();
    assert_eq!(search["searchCategoryType"], "Audio");
    assert_eq!(
        search["audioSearchFilters"]["audioTypes"],
        serde_json::json!(["Music"])
    );
    assert_eq!(search["audioSearchFilters"]["musicChartType"], "Month");
    assert_eq!(search["sortCategory"], "Top");
}

#[tokio::test]
async fn avatar_bundle_context_serializes_quantities_scale_and_colors() {
    let server = TestServer::start(vec![Reply::json(200, "{}")]);
    let client = AvatarBundleContextClient::new(transport(&server).await);
    client
        .create_context(CreateBundleContextInput {
            asset_quantities: Some(BTreeMap::from([("Head".into(), 1), ("Torso".into(), 1)])),
            bundle_description: Some("Description".into()),
            bundle_name: Some("Bundle".into()),
            bundle_type: Some(BundleType::Body),
            body_scale: Some(BodyScaleInput {
                height: Some(1.0),
                width: Some(0.9),
                head: Some(1.0),
                body_type: Some(0.5),
                proportion: Some(0.5),
            }),
            body_color_set: Some(BodyColorSetInput {
                head_color3: Some("ffffff".into()),
                torso_color3: Some("000000".into()),
                ..Default::default()
            }),
            group_id: Some(42),
            is_emissive: Some(false),
        })
        .await
        .unwrap();

    let requests = server.requests();
    assert_eq!(requests[0].target, "/v1/bundles/create-context");
    let body: serde_json::Value = serde_json::from_str(&requests[0].body_text()).unwrap();
    assert_eq!(body["assetQuantities"]["Head"], 1);
    assert_eq!(body["bundleType"], "Body");
    assert_eq!(body["bodyScale"]["width"], 0.9);
    assert_eq!(body["bodyColorSet"]["headColor3"], "ffffff");
}

#[tokio::test]
async fn folder_add_and_delete_send_typed_item_arrays() {
    let server = TestServer::start(vec![Reply::json(200, "{}"), Reply::json(200, "{}")]);
    let client = ItemFolderClient::new(transport(&server).await);
    let items = vec![FolderItemInput {
        id: Some("123".into()),
        item_type: Some(FolderItemType::Asset),
    }];
    client
        .add_items("folder".into(), items.clone())
        .await
        .unwrap();
    client.delete_items("folder".into(), items).await.unwrap();

    let requests = server.requests();
    assert_eq!(requests[0].target, "/v1/folders/folder/items");
    assert_eq!(requests[1].target, "/v1/folders/folder/items");
    assert_eq!(requests[0].method, "POST");
    assert_eq!(requests[1].method, "DELETE");
    assert_eq!(
        requests[0].body_text(),
        r#"{"items":[{"id":"123","itemType":"Asset"}]}"#
    );
    assert_eq!(requests[1].body_text(), requests[0].body_text());
}
