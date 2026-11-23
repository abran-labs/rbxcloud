mod support;
use rbxcloud::rbx::{
    campaign_planning::*,
    client::{AsyncClient, ClientPolicy},
    credentials::{Credentials, SecretString},
};
use std::time::Duration;
use support::{Reply, TestServer};

#[tokio::test]
async fn nested_campaign_context_uses_exact_json() {
    let server = TestServer::start(vec![Reply::json(200, "{}")]);
    let policy = ClientPolicy::loopback(server.addr())
        .unwrap()
        .timeout(Duration::from_secs(2))
        .retry_delay(Duration::from_millis(1));
    let client = CampaignPlanningClient::new(
        AsyncClient::new(
            Credentials::api_key(SecretString::new("campaign-planning-test-key")),
            policy,
        )
        .await
        .unwrap(),
    );
    client
        .recommend(PlanningContext {
            objective: CampaignObjective::Plays,
            target_universe_id: "123".into(),
            targeting: Some(PlanningTargeting {
                audience_type: Some(AudienceType::AllPlayers),
            }),
        })
        .await
        .unwrap();
    let requests = server.requests();
    assert_eq!(
        requests[0].target,
        "/ads-management/v1/campaign-recommendations"
    );
    assert_eq!(
        requests[0].body_text(),
        r#"{"campaignContext":{"objective":"PLAYS","targetUniverseId":"123","targeting":{"audienceType":"ALL_PLAYERS"}}}"#
    );
}
