mod support;

use std::time::Duration;

use rbxcloud::rbx::{
    analytics_alerts::*,
    analytics_dashboards::*,
    analytics_log_regexes::*,
    client::{AsyncClient, ClientPolicy},
    credentials::{Credentials, SecretString},
};
use support::{Reply, TestServer};

async fn transport(server: &TestServer) -> AsyncClient {
    let policy = ClientPolicy::loopback(server.addr())
        .unwrap()
        .timeout(Duration::from_secs(2))
        .retry_delay(Duration::from_millis(1));
    AsyncClient::new(
        Credentials::api_key(SecretString::new("analytics-management-test-key")),
        policy,
    )
    .await
    .unwrap()
}

fn document() -> CustomDashboardDocumentInput {
    CustomDashboardDocumentInput {
        schema_version: Some(1),
        config: Some(serde_json::json!({"page":{"untabbed":{"metadata":{"titleKey":"Overview"}}}})),
    }
}

#[tokio::test]
async fn dashboard_validate_create_metadata_and_publish_preserve_documents() {
    let server = TestServer::start((0..4).map(|_| Reply::json(200, "{}")).collect());
    let client = AnalyticsDashboardClient::new(transport(&server).await);
    client.validate_document(7, document()).await.unwrap();
    client
        .create_dashboard(
            7,
            CreateDashboardInput {
                name: Some("Overview".into()),
                description: Some("Daily".into()),
                document: Some(document()),
                idempotency_key: Some("key-1".into()),
            },
        )
        .await
        .unwrap();
    client
        .update_metadata(
            7,
            "dash".into(),
            UpdateDashboardMetadataInput {
                expected_head_etag: Some("etag".into()),
                patch: Some(DashboardMetadataPatchInput {
                    name: Some("Pinned".into()),
                    description: None,
                    is_pinned: Some(true),
                }),
            },
        )
        .await
        .unwrap();
    client
        .publish_dashboard(
            7,
            "dash".into(),
            PublishDashboardInput {
                expected_head_etag: Some("etag".into()),
                document: Some(document()),
                metadata_patch: Some(DashboardMetadataPatchInput {
                    name: None,
                    description: Some("Published".into()),
                    is_pinned: None,
                }),
            },
        )
        .await
        .unwrap();

    let requests = server.requests();
    assert_eq!(
        requests[0].target,
        "/analytics-custom-dashboards/v1/universes/7/custom-dashboard-documents/validate"
    );
    assert_eq!(
        requests[0].body_text(),
        r#"{"document":{"schemaVersion":1,"config":{"page":{"untabbed":{"metadata":{"titleKey":"Overview"}}}}}}"#
    );
    let create: serde_json::Value = serde_json::from_str(&requests[1].body_text()).unwrap();
    assert_eq!(create["idempotencyKey"], "key-1");
    assert_eq!(create["document"]["schemaVersion"], 1);
    assert_eq!(
        requests[2].target,
        "/analytics-custom-dashboards/v1/universes/7/custom-dashboards/dash/metadata"
    );
    let metadata: serde_json::Value = serde_json::from_str(&requests[2].body_text()).unwrap();
    assert_eq!(metadata["expectedHeadEtag"], "etag");
    assert_eq!(metadata["patch"]["isPinned"], true);
    assert_eq!(
        requests[3].target,
        "/analytics-custom-dashboards/v1/universes/7/custom-dashboards/dash/publish"
    );
    let publish: serde_json::Value = serde_json::from_str(&requests[3].body_text()).unwrap();
    assert_eq!(publish["metadataPatch"]["description"], "Published");
}

#[tokio::test]
async fn alert_create_and_update_serialize_conditions_filters_and_receivers() {
    let server = TestServer::start(vec![Reply::json(200, "{}"), Reply::json(200, "{}")]);
    let client = AnalyticsAlertClient::new(transport(&server).await);
    client
        .create_alert(
            "Universe",
            "7",
            CreateAlertInput {
                name: "Retention drop".into(),
                metric: "ForwardD1Retention".into(),
                description: None,
                severity: AlertSeverity::Sev1,
                interval: AlertInterval::OneDay,
                consecutive_occurrences: 2,
                breakdown: Some(vec![AlertBreakdownInput {
                    dimensions: vec!["platform".into()],
                }]),
                filter: Some(vec![AlertFilterInput {
                    dimension: "country".into(),
                    values: vec!["CA".into(), "US".into()],
                }]),
                condition: AlertConditionInput {
                    operator: Some(ConditionOperator::Lt),
                    threshold: Some(0.2),
                    evaluation_mode: Some(EvaluationMode::PeriodOverPeriod),
                    period_offset_multiplier: Some(7),
                },
                webhook_receiver_config: Some(WebhookReceiverConfigInput {
                    receivers: Some(vec![WebhookReceiverInput {
                        webhook_configuration_id: "webhook-id".into(),
                    }]),
                }),
            },
        )
        .await
        .unwrap();
    client
        .update_alert(
            "Universe",
            "7",
            "alert",
            UpdateAlertInput {
                name: Some("Paused".into()),
                config_state: Some(AlertConfigState::Disabled),
                condition: Some(AlertConditionInput {
                    operator: None,
                    threshold: Some(0.3),
                    evaluation_mode: None,
                    period_offset_multiplier: None,
                }),
                ..Default::default()
            },
        )
        .await
        .unwrap();

    let requests = server.requests();
    assert_eq!(
        requests[0].target,
        "/analytics-alert-control-plane/v1/resource/Universe/id/7/alerts"
    );
    let create: serde_json::Value = serde_json::from_str(&requests[0].body_text()).unwrap();
    assert_eq!(create["severity"], "SEV_1");
    assert_eq!(create["condition"]["operator"], "Lt");
    assert_eq!(create["condition"]["periodOffsetMultiplier"], 7);
    assert_eq!(
        create["filter"][0]["values"],
        serde_json::json!(["CA", "US"])
    );
    assert_eq!(
        create["webhookReceiverConfig"]["receivers"][0]["webhookConfigurationId"],
        "webhook-id"
    );
    assert_eq!(
        requests[1].target,
        "/analytics-alert-control-plane/v1/resource/Universe/id/7/alerts/alert"
    );
    let update: serde_json::Value = serde_json::from_str(&requests[1].body_text()).unwrap();
    assert_eq!(update["configState"], "Disabled");
    assert_eq!(update["condition"]["threshold"], 0.3);
}

#[tokio::test]
async fn analytics_log_regex_create_and_update_use_exact_operations() {
    let server = TestServer::start(vec![Reply::json(200, "{}"), Reply::json(200, "{}")]);
    let client = AnalyticsLogRegexClient::new(transport(&server).await);
    client
        .create_regex(CreateLogRegexInput {
            universe_id: 7,
            pattern: Some("error: (.*)".into()),
            output: Some("$1".into()),
            regex_operation: Some(RegexOperation::Group),
        })
        .await
        .unwrap();
    client
        .update_regex(
            "regex",
            UpdateLogRegexInput {
                pattern: Some("ignored".into()),
                output: None,
                regex_operation: Some(RegexOperation::Ignore),
            },
        )
        .await
        .unwrap();

    let requests = server.requests();
    assert_eq!(
        requests[0].target,
        "/log-attribute-http-service/v1/universe-regexes"
    );
    assert_eq!(
        requests[0].body_text(),
        r#"{"universeId":7,"pattern":"error: (.*)","output":"$1","regexOperation":"Group"}"#
    );
    assert_eq!(
        requests[1].target,
        "/log-attribute-http-service/v1/universe-regexes/regex"
    );
    assert_eq!(
        requests[1].body_text(),
        r#"{"pattern":"ignored","regexOperation":"Ignore"}"#
    );
}
