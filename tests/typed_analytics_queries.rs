mod support;
use rbxcloud::rbx::{
    analytics_queries::*,
    client::{AsyncClient, ClientPolicy},
    credentials::{Credentials, SecretString},
};
use std::time::Duration;
use support::{Reply, TestServer};

async fn client(server: &TestServer) -> AnalyticsQueryClient {
    let policy = ClientPolicy::loopback(server.addr())
        .unwrap()
        .timeout(Duration::from_secs(2))
        .retry_delay(Duration::from_millis(1));
    AnalyticsQueryClient::new(
        AsyncClient::new(
            Credentials::api_key(SecretString::new("analytics-query-test-key")),
            policy,
        )
        .await
        .unwrap(),
    )
}

#[tokio::test]
async fn metrics_dimensions_and_benchmarks_serialize_nested_filters() {
    let server = TestServer::start((0..3).map(|_| Reply::json(200, "{}")).collect());
    let client = client(&server).await;
    client
        .metrics(
            7,
            MetricsInput {
                metric: "Visits".into(),
                granularity: Granularity::OneDay,
                start_time: "2026-01-01T00:00:00Z".into(),
                end_time: "2026-01-02T00:00:00Z".into(),
                breakdown: Some(vec!["Platform".into()]),
                filter: Some(vec![QueryFilter {
                    dimension: "Country".into(),
                    operation: FilterOperation::In,
                    values: Some(vec!["CA".into(), "US".into()]),
                }]),
                limit: Some(10),
            },
        )
        .await
        .unwrap();
    client
        .dimensions(
            7,
            DimensionsInput {
                metric: "Visits".into(),
                dimensions: Some(vec!["Country".into()]),
                start_time: "a".into(),
                end_time: "b".into(),
                filter: None,
                granularity: Some(Granularity::OneHour),
                limit: None,
            },
        )
        .await
        .unwrap();
    client
        .benchmarks(
            7,
            BenchmarkInput {
                metric: Some("Visits".into()),
                start_time: Some("a".into()),
                end_time: Some("b".into()),
                filter: Some(vec![BreakdownFilter {
                    dimension: Some("Genre".into()),
                    value: Some("RPG".into()),
                }]),
                benchmark_type: Some(BenchmarkType::Genre),
                percentiles: Some(vec![50, 90]),
                benchmark_variant_id: None,
            },
        )
        .await
        .unwrap();
    let requests = server.requests();
    let metric: serde_json::Value = serde_json::from_str(&requests[0].body_text()).unwrap();
    assert_eq!(metric["granularity"], "OneDay");
    assert_eq!(metric["filter"][0]["operation"], "In");
    assert_eq!(
        metric["filter"][0]["values"],
        serde_json::json!(["CA", "US"])
    );
    assert_eq!(
        requests[0].target,
        "/analytics-query-api/v1/universes/7/metrics"
    );
    assert_eq!(
        requests[1].target,
        "/analytics-query-api/v1/universes/7/dimension-values"
    );
    assert_eq!(
        requests[2].target,
        "/analytics-benchmark/v1/universes/7/benchmarks"
    );
    let benchmark: serde_json::Value = serde_json::from_str(&requests[2].body_text()).unwrap();
    assert_eq!(benchmark["benchmarkType"], "BENCHMARK_TYPE_GENRE");
    assert_eq!(benchmark["filter"][0]["dimension"], "Genre");
}
