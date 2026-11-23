mod support;
use rbxcloud::rbx::{
    client::{AsyncClient, ClientPolicy},
    credentials::{Credentials, SecretString},
    matchmaking_attributes::*,
};
use std::time::Duration;
use support::{Reply, TestServer};

#[tokio::test]
async fn player_attributes_serialize_nested_values_and_locations() {
    let server = TestServer::start(vec![Reply::json(200, "{}"), Reply::json(200, "{}")]);
    let policy = ClientPolicy::loopback(server.addr())
        .unwrap()
        .timeout(Duration::from_secs(2))
        .retry_delay(Duration::from_millis(1));
    let client = MatchmakingAttributeClient::new(
        AsyncClient::new(
            Credentials::api_key(SecretString::new("matchmaking-attribute-test-key")),
            policy,
        )
        .await
        .unwrap(),
    );
    client
        .create(CreatePlayerAttributeInput {
            universe_id: 7,
            name: Some("Rank".into()),
            data_type: Some(AttributeDataType::Number),
            default_value: Some(AttributeValue {
                numeric_value: Some(5.0),
                value_type: Some(AttributeDataType::Number),
                ..Default::default()
            }),
            attribute_value_location: Some(AttributeLocation {
                location_case: "DataStoreLocation".into(),
                data_store_location: DataStoreLocationInput {
                    data_store_name: Some("Players".into()),
                    scope: Some("global".into()),
                    key_template: Some("rank-{UserId}".into()),
                    value_path: Some("rank".into()),
                },
            }),
        })
        .await
        .unwrap();
    client
        .update(
            "a".into(),
            UpdatePlayerAttributeInput {
                attribute_id: Some("a".into()),
                default_value: Some(AttributeValue {
                    string_value: Some("gold".into()),
                    value_type: Some(AttributeDataType::String),
                    ..Default::default()
                }),
                attribute_value_location: None,
            },
        )
        .await
        .unwrap();
    let requests = server.requests();
    let create: serde_json::Value = serde_json::from_str(&requests[0].body_text()).unwrap();
    assert_eq!(create["dataType"], "Number");
    assert_eq!(create["defaultValue"]["numericValue"], 5.0);
    assert_eq!(
        create["attributeValueLocation"]["dataStoreLocation"]["keyTemplate"],
        "rank-{UserId}"
    );
    assert_eq!(
        requests[1].target,
        "/matchmaking-api/v1/matchmaking/player-attribute/a"
    );
    assert_eq!(
        requests[1].body_text(),
        r#"{"attributeId":"a","defaultValue":{"stringValue":"gold","type":"String"}}"#
    );
}
