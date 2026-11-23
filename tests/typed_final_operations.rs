mod support;

use rbxcloud::rbx::{
    client::{AsyncClient, ClientPolicy},
    collectible_management::*,
    credentials::{Credentials, SecretString},
    place_instance_management::*,
    server_restarts::*,
    service_efficiency_configuration::*,
    speech_assets::*,
    virtual_event_management::*,
};
use std::{collections::BTreeMap, time::Duration};
use support::{Reply, TestServer};

async fn transport(server: &TestServer) -> AsyncClient {
    let policy = ClientPolicy::loopback(server.addr())
        .unwrap()
        .timeout(Duration::from_secs(2))
        .retry_delay(Duration::from_millis(1));
    AsyncClient::new(
        Credentials::api_key(SecretString::new("final-operation-test-key")),
        policy,
    )
    .await
    .unwrap()
}

#[tokio::test]
async fn collectible_publish_bulk_and_single_update_preserve_numeric_enums() {
    let server = TestServer::start((0..3).map(|_| Reply::json(200, "{}")).collect());
    let client = CollectibleClient::new(transport(&server).await);
    client
        .publish(PublishCollectibleInput {
            target_id: Some(123),
            target_type: Some(CollectibleTargetType::Asset),
            publishing_type: Some(CollectiblePublishingType::Limited),
            quantity: Some(100),
            price_in_robux: Some(50),
            resale_restriction: Some(CollectibleResaleRestriction::Disabled),
            sale_location_configuration: Some(CollectibleSaleLocationInput {
                sale_location_type: Some(CollectibleSaleLocationType::ShopAndExperiencesById),
                places: Some(vec![456]),
            }),
            ..Default::default()
        })
        .await
        .unwrap();
    client
        .bulk_update(BulkUpdateCollectiblesInput {
            idempotency_key: Some("bulk-1".into()),
            group_id: Some(42),
            update_targets: Some(BulkUpdateTargetsInput {
                asset_type: Some(vec![1, 8]),
                target: Some(vec![CollectibleTargetInput {
                    target_type: Some(CollectibleTargetType::Bundle),
                    target_id: Some("789".into()),
                }]),
            }),
            update_payload: Some(BulkUpdatePayloadInput {
                rental_opt_in: Some(true),
            }),
        })
        .await
        .unwrap();
    client
        .update(
            "collectible",
            UpdateCollectibleInput {
                sale_status: Some(CollectibleSaleStatus::OffSale),
                price_offset: Some(5),
                is_free: Some(false),
                ..Default::default()
            },
        )
        .await
        .unwrap();

    let requests = server.requests();
    let publish: serde_json::Value = serde_json::from_str(&requests[0].body_text()).unwrap();
    assert_eq!(publish["targetType"], 0);
    assert_eq!(publish["publishingType"], 1);
    assert_eq!(publish["resaleRestriction"], 2);
    assert_eq!(publish["saleLocationConfiguration"]["saleLocationType"], 4);
    assert_eq!(requests[1].target, "/v1/collectibles/bulk-update");
    let bulk: serde_json::Value = serde_json::from_str(&requests[1].body_text()).unwrap();
    assert_eq!(bulk["updateTargets"]["target"][0]["targetType"], 1);
    assert_eq!(bulk["updatePayload"]["rentalOptIn"], true);
    assert_eq!(requests[2].target, "/v1/collectibles/collectible");
    assert_eq!(
        requests[2].body_text(),
        r#"{"saleStatus":1,"priceOffset":5,"isFree":false}"#
    );
}

#[tokio::test]
async fn place_instance_update_serializes_engine_script_details() {
    let server = TestServer::start(vec![Reply::json(200, "{}")]);
    let client = PlaceInstanceClient::new(transport(&server).await);
    client
        .update(
            1,
            2,
            "instance",
            UpdatePlaceInstanceInput {
                path: Some("universes/1/places/2/instances/instance".into()),
                engine_instance: Some(EngineInstanceInput {
                    id: Some("engine-id".into()),
                    parent: None,
                    name: Some("Bootstrap".into()),
                    details: Some(EngineInstanceDetailsInput {
                        script: Some(EngineScriptInput {
                            enabled: Some(true),
                            run_context: Some(EngineRunContext::Server),
                            source: Some("print('ok')".into()),
                        }),
                        ..Default::default()
                    }),
                }),
            },
        )
        .await
        .unwrap();
    let requests = server.requests();
    assert_eq!(
        requests[0].target,
        "/cloud/v2/universes/1/places/2/instances/instance"
    );
    let body: serde_json::Value = serde_json::from_str(&requests[0].body_text()).unwrap();
    assert_eq!(body["engineInstance"]["Name"], "Bootstrap");
    assert_eq!(
        body["engineInstance"]["Details"]["Script"]["RunContext"],
        "Server"
    );
    assert_eq!(
        body["engineInstance"]["Details"]["Script"]["Source"],
        "print('ok')"
    );
}

#[tokio::test]
async fn speech_generation_sends_text_and_voice_style() {
    let server = TestServer::start(vec![Reply::json(200, "{}")]);
    let client = SpeechAssetClient::new(transport(&server).await);
    client
        .generate(
            7,
            GenerateSpeechInput {
                text: "Hello".into(),
                speech_style: Some(SpeechStyleInput {
                    voice_id: Some("1".into()),
                    pitch: Some(1.2),
                    speed: Some(0.8),
                }),
            },
        )
        .await
        .unwrap();
    let requests = server.requests();
    assert_eq!(
        requests[0].target,
        "/cloud/v2/universes/7:generateSpeechAsset"
    );
    assert_eq!(
        requests[0].body_text(),
        r#"{"text":"Hello","speechStyle":{"voiceId":"1","pitch":1.2,"speed":0.8}}"#
    );
}

#[tokio::test]
async fn server_restart_serializes_place_filters_and_attributes() {
    let server = TestServer::start(vec![Reply::json(200, "{}")]);
    let client = ServerRestartClient::new(transport(&server).await);
    client
        .restart(
            7,
            LaunchRestartInput {
                bleed_off_duration_minutes: Some(15),
                places: Some(BTreeMap::from([(
                    "123".into(),
                    PlaceVersionFilterInput {
                        versions: Some(vec![4, 5]),
                        exclude_current_version: None,
                    },
                )])),
                attributes: Some(serde_json::json!({"reason":"deploy"})),
            },
        )
        .await
        .unwrap();
    let requests = server.requests();
    assert_eq!(
        requests[0].target,
        "/server-management/v1/universes/7/restarts"
    );
    let body: serde_json::Value = serde_json::from_str(&requests[0].body_text()).unwrap();
    assert_eq!(body["places"]["123"]["versions"], serde_json::json!([4, 5]));
    assert_eq!(body["attributes"]["reason"], "deploy");
}

#[tokio::test]
async fn service_efficiency_update_serializes_nested_money_and_prices() {
    let server = TestServer::start(vec![Reply::json(200, "{}")]);
    let client = ServiceEfficiencyClient::new(transport(&server).await);
    client
        .update(
            7,
            vec![ServiceConfigurationInput {
                service_id: "memory-store".into(),
                monthly_budget: Some(ServiceMoneyInput {
                    currency_code: Some("USD".into()),
                    units: Some(10),
                    nanos: Some(0),
                }),
                resource_configurations: vec![ResourceConfigurationInput {
                    resource_id: "requests".into(),
                    unlocked: true,
                    monthly_budget: None,
                    unit_cost: ServiceMoneyInput {
                        currency_code: Some("USD".into()),
                        units: Some(0),
                        nanos: Some(4_400_000),
                    },
                    price: ServicePriceInput {
                        cost: Some(ServiceMoneyInput {
                            currency_code: Some("USD".into()),
                            units: Some(0),
                            nanos: Some(4_400_000),
                        }),
                        unit_amount: Some(1_000_000),
                    },
                }],
                budget_level: ServiceBudgetLevel::ResourceOnly,
            }],
        )
        .await
        .unwrap();
    let requests = server.requests();
    assert_eq!(
        requests[0].target,
        "/service-efficiency-api/v1/configurations/universes/7"
    );
    let body: serde_json::Value = serde_json::from_str(&requests[0].body_text()).unwrap();
    assert_eq!(
        body["serviceConfigurations"][0]["budgetLevel"],
        "ResourceOnly"
    );
    assert_eq!(
        body["serviceConfigurations"][0]["resourceConfigurations"][0]["price"]["unitAmount"],
        1_000_000
    );
}

#[tokio::test]
async fn virtual_event_create_and_update_use_typed_categories_and_media() {
    let server = TestServer::start(vec![Reply::json(200, "{}"), Reply::json(200, "{}")]);
    let client = VirtualEventClient::new(transport(&server).await);
    client
        .create(
            7,
            CreateVirtualEventInput {
                title: Some("Launch".into()),
                subtitle: Some("New season".into()),
                description: Some("Play".into()),
                start_time: Some("2027-01-01T00:00:00Z".into()),
                end_time: Some("2027-01-02T00:00:00Z".into()),
                visibility: Some(EventVisibilityInput::Public),
                group_id: Some(42),
                place_id: Some(123),
                categories: Some(vec![EventRankedCategoryInput {
                    category: Some(EventCategoryInput::NewSeason),
                    rank: Some(1),
                }]),
                thumbnails: Some(vec![EventMediaInput {
                    media_id: Some(456),
                    rank: Some(1),
                }]),
                featuring_status: Some(EventFeaturingStatus::Enabled),
                tagline: Some("Join".into()),
            },
        )
        .await
        .unwrap();
    client
        .update(
            "event",
            UpdateVirtualEventInput {
                visibility: Some(EventVisibilityInput::Private),
                featuring_status: Some(EventFeaturingStatus::Disabled),
                tagline: Some("Updated".into()),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    let requests = server.requests();
    assert_eq!(
        requests[0].target,
        "/virtual-events/v3/universes/7/game-events"
    );
    let create: serde_json::Value = serde_json::from_str(&requests[0].body_text()).unwrap();
    assert_eq!(create["visibility"], "public");
    assert_eq!(create["categories"][0]["category"], "newSeason");
    assert_eq!(create["featuringStatus"], "enabled");
    assert_eq!(requests[1].target, "/virtual-events/v3/game-events/event");
    assert_eq!(
        requests[1].body_text(),
        r#"{"visibility":"private","featuringStatus":"disabled","tagline":"Updated"}"#
    );
}
