mod support;
use rbxcloud::rbx::{
    client::{AsyncClient, ClientPolicy},
    credentials::{Credentials, SecretString},
    generated::{composed::*, composed_models::*},
};
use std::time::Duration;
use support::{Reply, TestServer};

async fn client(server: &TestServer) -> ComposedCloudClient {
    let policy = ClientPolicy::loopback(server.addr())
        .unwrap()
        .timeout(Duration::from_secs(2))
        .retry_delay(Duration::from_millis(1));
    ComposedCloudClient::new(
        AsyncClient::new(
            Credentials::api_key(SecretString::new("composed-contract-test-key")),
            policy,
        )
        .await
        .unwrap(),
    )
}

#[tokio::test]
async fn composed_bodies_keep_required_fields_and_wire_names() {
    let server = TestServer::start(
        (0..3)
            .map(|_| Reply::json(400, r#"{"message":"contract stop"}"#))
            .collect(),
    );
    let client = client(&server).await;
    let _ = client
        .experience_notifications_create_notification(
            ExperienceNotificationsCreateNotificationParams {
                body: ExperienceNotificationsCreateNotificationRequest {
                    name: "Update".into(),
                    content: "Hello".into(),
                    universe_id: 7,
                },
            },
        )
        .await;
    let _ = client
        .place_versions_update_place_version_notes(PlaceVersionsUpdatePlaceVersionNotesParams {
            place_id: 1,
            version: "2".into(),
            body: PlaceVersionsUpdatePlaceVersionNotesRequest {
                title: Some("Title".into()),
                description: Some("Description".into()),
            },
        })
        .await;
    let _ = client
        .analytics_logs_reorder_log_regex(AnalyticsLogsReorderLogRegexParams {
            id: 3,
            body: AnalyticsLogsReorderLogRegexRequest { order: Some(4) },
        })
        .await;
    let requests = server.requests();
    assert_eq!(
        requests[0].body_text(),
        r#"{"name":"Update","content":"Hello","universeId":7}"#
    );
    assert_eq!(
        requests[1].target,
        "/place-version-history-api/v1/1/version/2/notes"
    );
    assert_eq!(
        requests[1].body_text(),
        r#"{"title":"Title","description":"Description"}"#
    );
    assert_eq!(
        requests[2].target,
        "/log-attribute-http-service/v1/universe-regexes/3/reorder"
    );
    assert_eq!(requests[2].body_text(), r#"{"order":4}"#);
}
