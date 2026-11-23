mod support;
use rbxcloud::rbx::{
    client::{AsyncClient, ClientPolicy},
    config_publishing::ConditionalRuleDefinitionInput,
    credentials::{Credentials, SecretString},
    experiment_management::*,
};
use std::time::Duration;
use support::{Reply, TestServer};

#[tokio::test]
async fn experiment_configuration_targeting_and_mde_fields_serialize() {
    let server = TestServer::start((0..3).map(|_| Reply::json(200, "{}")).collect());
    let policy = ClientPolicy::loopback(server.addr())
        .unwrap()
        .timeout(Duration::from_secs(2))
        .retry_delay(Duration::from_millis(1));
    let client = ExperimentClient::new(
        AsyncClient::new(
            Credentials::api_key(SecretString::new("experiment-management-test-key")),
            policy,
        )
        .await
        .unwrap(),
    );
    let input = ExperimentInput {
        name: Some("Test".into()),
        description: None,
        experiment_configuration: Some(ExperimentConfigurationInput {
            product_type: Some(ExperimentProductType::InGameConfigs),
            ..Default::default()
        }),
        exposure_percent: Some(50),
        targeting_criteria: Some(TargetingCriteriaInput {
            rule: Some(ConditionalRuleDefinitionInput {
                tokens: Some(vec![]),
            }),
        }),
        duration_seconds: Some(86400),
        universe_goal_metric: Some(GoalMetric::Day1Retention),
    };
    client.create(7, input).await.unwrap();
    client
        .update(
            7,
            "e".into(),
            ExperimentInput {
                name: Some("Updated".into()),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    client
        .calculate_mde(
            7,
            MdeInput {
                exposure_percent: Some(50),
                duration_seconds: Some(86400),
                baseline_weight: Some(50.0),
                variant_weights: Some(vec![25.0, 25.0]),
                universe_goal_metric: Some(GoalMetric::Day7Retention),
                targeting_criteria: None,
            },
        )
        .await
        .unwrap();
    let requests = server.requests();
    let create: serde_json::Value = serde_json::from_str(&requests[0].body_text()).unwrap();
    assert_eq!(
        create["universeGoalMetric"],
        "UNIVERSE_EXPERIMENT_METRIC_DAY_1_RETENTION"
    );
    assert_eq!(
        create["targetingCriteria"]["rule"]["tokens"],
        serde_json::json!([])
    );
    assert_eq!(
        requests[1].target,
        "/creator-configs-public-api/v1/experimentation/universes/7/experiments/e"
    );
    assert_eq!(requests[1].body_text(), r#"{"name":"Updated"}"#);
    let mde: serde_json::Value = serde_json::from_str(&requests[2].body_text()).unwrap();
    assert_eq!(mde["variantWeights"], serde_json::json!([25.0, 25.0]));
    assert_eq!(
        mde["universeGoalMetric"],
        "UNIVERSE_EXPERIMENT_METRIC_DAY_7_RETENTION"
    );
}
