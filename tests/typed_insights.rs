mod support;
use rbxcloud::rbx::{
    client::{AsyncClient, ClientPolicy},
    credentials::{Credentials, SecretString},
    generated::{insights::*, insights_models::*},
};
use std::time::Duration;
use support::{Reply, TestServer};

async fn client(server: &TestServer) -> InsightsCloudClient {
    let policy = ClientPolicy::loopback(server.addr())
        .unwrap()
        .timeout(Duration::from_secs(2))
        .retry_delay(Duration::from_millis(1));
    InsightsCloudClient::new(
        AsyncClient::new(
            Credentials::api_key(SecretString::new("insights-contract-test-key")),
            policy,
        )
        .await
        .unwrap(),
    )
}

#[tokio::test]
async fn insights_queries_use_exact_repeated_arrays_and_paths() {
    let server = TestServer::start(vec![
        Reply::json(200, "{}"),
        Reply::json(200, "{\"ads\":[]}"),
        Reply::json(200, "[]"),
        Reply::json(200, "{}"),
        Reply::json(200, "{}"),
        Reply::json(200, "{}"),
    ]);
    let client = client(&server).await;
    client
        .list_campaigns(ListCampaignsParams {
            group_id: None,
            target_universe_ids: Some(vec!["1".into(), "2".into()]),
            ended_after: None,
            max_page_size: Some(10),
            page_token: None,
        })
        .await
        .unwrap();
    client
        .list_campaign_ads(ListCampaignAdsParams {
            campaign_id: "c".into(),
            max_page_size: Some(5),
            page_token: None,
        })
        .await
        .unwrap();
    client
        .list_alerts(ListAlertsParams {
            resource_type: ListAlertsResourceType::Universe,
            resource_id: "1".into(),
            ids: Some(vec!["a".into(), "b".into()]),
            firing_status: None,
            severities: Some(vec![AlertStorageSeverity::Sev2]),
            metrics: Some(vec!["players".into()]),
        })
        .await
        .unwrap();
    client
        .list_dashboards(ListDashboardsParams {
            universe_id: 1,
            page_size: Some(4),
            page_token: None,
        })
        .await
        .unwrap();
    client
        .get_metrics_operation(GetMetricsOperationParams {
            universe_id: 1,
            operation_id: "o".into(),
        })
        .await
        .unwrap();
    client
        .get_error_log_details(GetErrorLogDetailsParams {
            universe_id: 1,
            message_hash_ids: Some(vec!["h1".into(), "h2".into()]),
            keyword: None,
            start_time: None,
            end_time: None,
            place_id: None,
            place_versions: Some(vec![3, 4]),
            first_seen_place_version: None,
        })
        .await
        .unwrap();
    let requests = server.requests();
    assert_eq!(
        requests[0].target,
        "/ads-management/v1/campaigns?maxPageSize=10&targetUniverseIds=1&targetUniverseIds=2"
    );
    assert_eq!(
        requests[1].target,
        "/ads-management/v1/campaigns/c/ads?maxPageSize=5"
    );
    assert_eq!(requests[2].target, "/analytics-alert-control-plane/v1/resource/Universe/id/1/alerts?ids=a&ids=b&severities=SEV_2&metrics=players");
    assert_eq!(
        requests[3].target,
        "/analytics-custom-dashboards/v1/universes/1/custom-dashboards?pageSize=4"
    );
    assert_eq!(
        requests[4].target,
        "/analytics-query-api/v1/universes/1/operations/metrics/o"
    );
    assert_eq!(requests[5].target, "/universe-performance-raqi/v1/events/logging/universe/1/log-details?messageHashIds=h1&messageHashIds=h2&placeVersions=3&placeVersions=4");
}

#[test]
fn insights_cli_has_explicit_subgroups() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_rbxcloud"))
        .args(["insights-tools", "--help"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let help = String::from_utf8(output.stdout).unwrap();
    for group in [
        "advertising",
        "analytics-alerts",
        "analytics-dashboards",
        "analytics-logs",
        "analytics-queries",
    ] {
        assert!(help.contains(group));
    }
}
