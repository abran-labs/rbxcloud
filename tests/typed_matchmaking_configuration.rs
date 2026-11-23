mod support;

use std::{collections::BTreeMap, time::Duration};

use rbxcloud::rbx::{
    client::{AsyncClient, ClientPolicy},
    credentials::{Credentials, SecretString},
    matchmaking_attributes::{AttributeDataType, AttributeValue},
    matchmaking_configuration::*,
};
use support::{Reply, TestServer};

async fn client(server: &TestServer) -> MatchmakingConfigurationClient {
    let policy = ClientPolicy::loopback(server.addr())
        .unwrap()
        .timeout(Duration::from_secs(2))
        .retry_delay(Duration::from_millis(1));
    MatchmakingConfigurationClient::new(
        AsyncClient::new(
            Credentials::api_key(SecretString::new("matchmaking-configuration-test-key")),
            policy,
        )
        .await
        .unwrap(),
    )
}

#[tokio::test]
async fn server_attributes_serialize_constant_and_player_reference_defaults() {
    let server = TestServer::start(vec![Reply::json(200, "{}"), Reply::json(200, "{}")]);
    let client = client(&server).await;
    client
        .create_server_attribute(CreateServerAttributeInput {
            universe_id: 7,
            name: Some("Region".into()),
            data_type: Some(AttributeDataType::String),
            default_value: Some(ServerDefaultValueInput {
                source_case: Some(ServerDefaultSource::Constant),
                constant: Some(AttributeValue {
                    string_value: Some("us-east".into()),
                    value_type: Some(AttributeDataType::String),
                    ..Default::default()
                }),
                player_attribute_reference: None,
            }),
        })
        .await
        .unwrap();
    client
        .update_server_attribute(
            "server-attr".into(),
            UpdateServerAttributeInput {
                attribute_id: Some("server-attr".into()),
                default_value: Some(ServerDefaultValueInput {
                    source_case: Some(ServerDefaultSource::PlayerAttributeReference),
                    constant: None,
                    player_attribute_reference: Some(AttributeReferenceInput {
                        id: Some("player-attr".into()),
                    }),
                }),
            },
        )
        .await
        .unwrap();

    let requests = server.requests();
    let create: serde_json::Value = serde_json::from_str(&requests[0].body_text()).unwrap();
    assert_eq!(create["defaultValue"]["sourceCase"], "Constant");
    assert_eq!(create["defaultValue"]["constant"]["stringValue"], "us-east");
    assert_eq!(
        requests[1].target,
        "/matchmaking-api/v1/matchmaking/server-attribute/server-attr"
    );
    assert_eq!(
        requests[1].body_text(),
        r#"{"attributeId":"server-attr","defaultValue":{"sourceCase":"PlayerAttributeReference","playerAttributeReference":{"id":"player-attr"}}}"#
    );
}

#[tokio::test]
async fn scoring_configurations_serialize_standard_and_custom_weights() {
    let server = TestServer::start(vec![Reply::json(200, "{}"), Reply::json(200, "{}")]);
    let client = client(&server).await;
    let custom = BTreeMap::from([("Skill".into(), 2.5)]);
    client
        .create_scoring_configuration(CreateScoringConfigurationInput {
            universe_id: 7,
            name: Some("Ranked".into()),
            matchmaking_signal_weights: Some(SignalWeightsInput {
                occupancy: Some(1.0),
                latency: Some(-0.5),
                ..Default::default()
            }),
            custom_signal_weights: Some(custom.clone()),
            template: Some(ScoringTemplate::NotApplicable),
        })
        .await
        .unwrap();
    client
        .update_scoring_configuration(
            "config".into(),
            UpdateScoringConfigurationInput {
                scoring_configuration_id: Some("config".into()),
                name: Some("Updated".into()),
                matchmaking_signal_weights: Some(SignalWeightsInput {
                    preferred_players: Some(3.0),
                    ..Default::default()
                }),
                custom_signal_weights: Some(custom),
            },
        )
        .await
        .unwrap();

    let requests = server.requests();
    let create: serde_json::Value = serde_json::from_str(&requests[0].body_text()).unwrap();
    assert_eq!(create["matchmakingSignalWeights"]["Occupancy"], 1.0);
    assert_eq!(create["matchmakingSignalWeights"]["Latency"], -0.5);
    assert_eq!(create["customSignalWeights"]["Skill"], 2.5);
    assert_eq!(create["template"], "NotApplicable");
    assert_eq!(
        requests[1].target,
        "/matchmaking-api/v1/matchmaking/scoring-configuration/config"
    );
    let update: serde_json::Value = serde_json::from_str(&requests[1].body_text()).unwrap();
    assert_eq!(update["matchmakingSignalWeights"]["PreferredPlayers"], 3.0);
}

#[tokio::test]
async fn custom_signals_serialize_every_configuration_variant() {
    let server = TestServer::start((0..4).map(|_| Reply::json(200, "{}")).collect());
    let client = client(&server).await;
    let reference = |id: &str| {
        Some(AttributeReferenceInput {
            id: Some(id.into()),
        })
    };
    let variants = [
        CustomSignalInput {
            name: Some("Language".into()),
            custom_signal_type: Some(CustomSignalType::PlayerCategorical),
            player_categorical_signal_configuration: Some(PlayerCategoricalSignalInput {
                player_attribute: reference("language"),
                curve_type: Some(SignalCurveType::PositiveLinear),
            }),
            ..Default::default()
        },
        CustomSignalInput {
            name: Some("Region".into()),
            custom_signal_type: Some(CustomSignalType::ServerCategorical),
            server_categorical_signal_configuration: Some(ServerCategoricalSignalInput {
                server_attribute: reference("region"),
                comparison_type: Some(CategoricalComparison::EqualityToConstant),
                player_attribute: None,
                constant_value: Some("us-east".into()),
            }),
            ..Default::default()
        },
        CustomSignalInput {
            name: Some("Occupancy".into()),
            custom_signal_type: Some(CustomSignalType::ServerNumerical),
            server_numerical_signal_configuration: Some(ServerNumericalSignalInput {
                server_attribute: reference("occupancy"),
                comparison_type: Some(NumericalComparison::AbsoluteDifferentToConstant),
                max_relevant_difference: Some(10.0),
                constant_value: Some(4.0),
                player_attribute: None,
            }),
            ..Default::default()
        },
        CustomSignalInput {
            name: Some("Skill".into()),
            custom_signal_type: Some(CustomSignalType::PlayerNumerical),
            player_numerical_signal_configuration: Some(PlayerNumericalSignalInput {
                player_attribute: reference("skill"),
                aggregation: Some(AttributeAggregationInput {
                    aggregation_function: Some(AggregationFunction::Average),
                }),
                comparison_type: Some(NumericalComparison::AbsoluteDifferentToPlayer),
                max_relevant_difference: Some(100.0),
                constant_value: None,
            }),
            ..Default::default()
        },
    ];
    let mut variants = variants.into_iter();
    for input in variants.by_ref().take(3) {
        client
            .create_custom_signal(
                "config".into(),
                CreateCustomSignalInput {
                    scoring_configuration_id: Some("config".into()),
                    signal_configuration: Some(input),
                },
            )
            .await
            .unwrap();
    }
    client
        .update_custom_signal(
            "config".into(),
            "Skill".into(),
            UpdateCustomSignalInput {
                scoring_configuration_id: Some("config".into()),
                signal_configuration: variants.next(),
            },
        )
        .await
        .unwrap();

    let requests = server.requests();
    assert_eq!(
        requests[0].target,
        "/matchmaking-api/v1/matchmaking/scoring-configuration/config/signals"
    );
    let player_cat: serde_json::Value = serde_json::from_str(&requests[0].body_text()).unwrap();
    assert_eq!(
        player_cat["signalConfiguration"]["playerCategoricalSignalConfiguration"]["curveType"],
        "PositiveLinear"
    );
    let server_cat: serde_json::Value = serde_json::from_str(&requests[1].body_text()).unwrap();
    assert_eq!(
        server_cat["signalConfiguration"]["serverCategoricalSignalConfiguration"]["constantValue"],
        "us-east"
    );
    let server_num: serde_json::Value = serde_json::from_str(&requests[2].body_text()).unwrap();
    assert_eq!(
        server_num["signalConfiguration"]["serverNumericalSignalConfiguration"]
            ["maxRelevantDifference"],
        10.0
    );
    assert_eq!(
        requests[3].target,
        "/matchmaking-api/v1/matchmaking/scoring-configuration/config/signals/Skill"
    );
    let player_num: serde_json::Value = serde_json::from_str(&requests[3].body_text()).unwrap();
    assert_eq!(
        player_num["signalConfiguration"]["playerNumericalSignalConfiguration"]["aggregation"]
            ["aggregationFunction"],
        "Average"
    );
}
