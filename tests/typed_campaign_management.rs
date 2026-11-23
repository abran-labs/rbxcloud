mod support;

use std::time::Duration;

use rbxcloud::rbx::{
    campaign_management::{
        AudienceEstimateInput, BudgetInput, BudgetType, CampaignClient, CampaignInput,
        CampaignStatus, CampaignUpdateInput, Device, PaymentType, PerformanceEstimateInput,
        PerformanceForecastInput, PerformanceObjective, PerformanceScope, PerformanceTargetOutcome,
        Region, ScheduleInput, TargetingInput, UpdateBudgetInput, UpdateScheduleInput, VideoInput,
        VideoSource,
    },
    campaign_planning::{AudienceType, CampaignObjective},
    client::{AsyncClient, ClientPolicy},
    credentials::{Credentials, SecretString},
};
use support::{Reply, TestServer};

async fn client(server: &TestServer) -> CampaignClient {
    let policy = ClientPolicy::loopback(server.addr())
        .unwrap()
        .timeout(Duration::from_secs(2))
        .retry_delay(Duration::from_millis(1));
    CampaignClient::new(
        AsyncClient::new(
            Credentials::api_key(SecretString::new("campaign-management-test-key")),
            policy,
        )
        .await
        .unwrap(),
    )
}

#[tokio::test]
async fn audience_estimate_serializes_full_targeting() {
    let server = TestServer::start(vec![Reply::json(200, "{}")]);
    client(&server)
        .await
        .estimate_audience(AudienceEstimateInput {
            objective: CampaignObjective::Plays,
            target_universe_id: "123".into(),
            targeting: Some(TargetingInput {
                audience_type: Some(AudienceType::NewUsers),
                countries: Some(vec!["CA".into(), "US".into()]),
                devices: Some(vec![Device::Phone, Device::Desktop]),
                regions: Some(vec![Region::Anz]),
                ..Default::default()
            }),
        })
        .await
        .unwrap();

    let requests = server.requests();
    assert_eq!(requests[0].target, "/ads-management/v1/audience-estimates");
    assert_eq!(
        requests[0].body_text(),
        r#"{"campaignContext":{"objective":"PLAYS","targetUniverseId":"123","targeting":{"audienceType":"NEW_USERS","countries":["CA","US"],"devices":["PHONE","DESKTOP"],"regions":["ANZ"]}}}"#
    );
}

#[tokio::test]
async fn campaign_create_serializes_nested_budget_schedule_and_video() {
    let server = TestServer::start(vec![Reply::json(200, "{}")]);
    client(&server)
        .await
        .create(CampaignInput {
            billing_account_id: Some("billing-1".into()),
            budget: Some(BudgetInput {
                amount_micros: "5000000".into(),
                budget_type: BudgetType::Daily,
            }),
            schedule: Some(ScheduleInput {
                duration_in_days: 7,
                start_time: "2027-01-01T00:00:00Z".into(),
            }),
            creative_asset_ids: Some(vec!["asset-1".into()]),
            idempotency_key: Some("00000000-0000-0000-0000-000000000001".into()),
            objective: Some(CampaignObjective::RobuxSpend),
            payment_type: Some(PaymentType::AdsCredit),
            target_universe_id: Some("123".into()),
            video: Some(VideoInput {
                source: VideoSource::PreviewVideo,
            }),
            ..Default::default()
        })
        .await
        .unwrap();

    let requests = server.requests();
    assert_eq!(requests[0].target, "/ads-management/v1/campaigns");
    let body: serde_json::Value = serde_json::from_str(&requests[0].body_text()).unwrap();
    assert_eq!(body["objective"], "ROBUX_SPEND");
    assert_eq!(body["creativeAssetIds"], serde_json::json!(["asset-1"]));
    assert_eq!(
        body["idempotencyKey"],
        "00000000-0000-0000-0000-000000000001"
    );
    assert!(body.get("status").is_none());
    assert_eq!(body["budget"]["amountMicros"], "5000000");
    assert_eq!(body["budget"]["type"], "DAILY");
    assert_eq!(body["schedule"]["durationInDays"], 7);
    assert_eq!(body["video"]["source"], "PREVIEW_VIDEO");
}

#[tokio::test]
async fn campaign_validate_uses_validation_route_without_mutating_create_route() {
    let server = TestServer::start(vec![Reply::json(200, "{}")]);
    client(&server)
        .await
        .validate(CampaignInput {
            name: Some("Validation only".into()),
            ..Default::default()
        })
        .await
        .unwrap();

    let requests = server.requests();
    assert_eq!(requests[0].target, "/ads-management/v1/campaigns:validate");
    assert_eq!(requests[0].body_text(), r#"{"name":"Validation only"}"#);
}

#[tokio::test]
async fn campaign_update_omits_immutable_budget_type_and_accepts_video() {
    let server = TestServer::start(vec![Reply::json(200, "{}")]);
    client(&server)
        .await
        .update(
            "campaign-1".into(),
            CampaignUpdateInput {
                budget: Some(UpdateBudgetInput {
                    amount_micros: "7000000".into(),
                }),
                schedule: Some(UpdateScheduleInput {
                    duration_in_days: Some(14),
                    start_time: None,
                }),
                status: Some(CampaignStatus::Paused),
                video: Some(VideoInput {
                    source: VideoSource::ImageOnly,
                }),
                ..Default::default()
            },
        )
        .await
        .unwrap();

    let requests = server.requests();
    assert_eq!(
        requests[0].target,
        "/ads-management/v1/campaigns/campaign-1"
    );
    assert_eq!(
        requests[0].body_text(),
        r#"{"budget":{"amountMicros":"7000000"},"schedule":{"durationInDays":14},"status":"PAUSED","video":{"source":"IMAGE_ONLY"}}"#
    );
}

#[tokio::test]
async fn performance_estimate_serializes_target_outcome_and_scope() {
    let server = TestServer::start(vec![Reply::json(200, "{}")]);
    client(&server)
        .await
        .estimate_performance(PerformanceEstimateInput {
            objective: PerformanceObjective::Plays,
            target_universe_id: "123".into(),
            forecast_input: PerformanceForecastInput {
                target_outcome: Some(PerformanceTargetOutcome {
                    outcome_type: PerformanceObjective::Plays,
                    value: Some(1000),
                }),
                ..Default::default()
            },
            forecast_scope: PerformanceScope {
                budget_type: BudgetType::Lifetime,
                duration_in_days: Some(30),
            },
        })
        .await
        .unwrap();

    let requests = server.requests();
    assert_eq!(
        requests[0].target,
        "/ads-management/v1/performance-estimates"
    );
    assert_eq!(
        requests[0].body_text(),
        r#"{"campaignContext":{"objective":"PLAYS","targetUniverseId":"123"},"forecastInput":{"targetOutcome":{"outcomeType":"PLAYS","value":1000}},"forecastScope":{"budgetType":"LIFETIME","durationInDays":30}}"#
    );
}
