mod support;
use rbxcloud::rbx::{
    client::{AsyncClient, ClientPolicy},
    config_publishing::*,
    credentials::{Credentials, SecretString},
};
use std::time::Duration;
use support::{Reply, TestServer};

#[tokio::test]
async fn draft_and_publish_bodies_keep_typed_maps_and_enums() {
    let server = TestServer::start((0..3).map(|_| Reply::json(200, "{}")).collect());
    let policy = ClientPolicy::loopback(server.addr())
        .unwrap()
        .timeout(Duration::from_secs(2))
        .retry_delay(Duration::from_millis(1));
    let client = ConfigPublishingClient::new(
        AsyncClient::new(
            Credentials::api_key(SecretString::new("config-publish-test-key")),
            policy,
        )
        .await
        .unwrap(),
    );
    let update = DraftUpdateInput {
        draft_hash: Some("hash".into()),
        entries: Some(serde_json::json!({"BossHealth":100})),
        conditional_rules: Some(ConditionalRulesInput {
            rules: Some(serde_json::json!({"vip":{"tokens":[]}})),
            rules_order: Some(vec!["vip".into()]),
        }),
    };
    client
        .update("1".into(), "repo".into(), update.clone())
        .await
        .unwrap();
    client
        .overwrite("1".into(), "repo".into(), update)
        .await
        .unwrap();
    client
        .publish(
            "1".into(),
            "repo".into(),
            DraftPublishInput {
                draft_hash: Some("hash".into()),
                message: Some("Release".into()),
                deployment_strategy: Some(DeploymentStrategy::Immediate),
            },
        )
        .await
        .unwrap();
    let requests = server.requests();
    let body: serde_json::Value = serde_json::from_str(&requests[0].body_text()).unwrap();
    assert_eq!(body["draftHash"], "hash");
    assert_eq!(body["entries"]["BossHealth"], 100);
    assert_eq!(
        body["conditionalRules"]["rulesOrder"],
        serde_json::json!(["vip"])
    );
    assert!(requests[1].target.ends_with("/draft:overwrite"));
    let publish: serde_json::Value = serde_json::from_str(&requests[2].body_text()).unwrap();
    assert_eq!(publish["deploymentStrategy"], "Immediate");
}
