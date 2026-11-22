mod support;

use rbxcloud::rbx::{
    client::{AsyncClient, ClientPolicy},
    cloud::{
        CloudClient, DataStoreEntry, DataStoreEntryPath, ListDataStoresParams,
        SnapshotDataStoresRequest,
    },
    credentials::{Credentials, SecretString},
};
use serde_json::json;
use std::time::Duration;
use support::{Reply, TestServer};

const KEY: &str = "typed-storage-test-key-not-a-real-key";

fn path() -> DataStoreEntryPath {
    DataStoreEntryPath {
        universe_id: "123".into(),
        data_store_id: "player data".into(),
        entry_id: "user/42".into(),
    }
}

async fn client(server: &TestServer) -> CloudClient {
    let policy = ClientPolicy::loopback(server.addr())
        .unwrap()
        .timeout(Duration::from_secs(2))
        .retry_delay(Duration::from_millis(1));
    CloudClient::new(
        AsyncClient::new(Credentials::api_key(SecretString::new(KEY)), policy)
            .await
            .unwrap(),
    )
}

#[tokio::test]
async fn list_data_stores_uses_exact_path_and_query() {
    let server = TestServer::start(vec![Reply::json(
        200,
        r#"{"dataStores":[{"path":"universes/123/data-stores/players","id":"players"}],"nextPageToken":"next"}"#,
    )]);
    let result = client(&server)
        .await
        .list_data_stores(ListDataStoresParams {
            universe_id: "123".into(),
            max_page_size: Some(50),
            page_token: Some("cursor".into()),
            filter: Some("id.startsWith(\"players\")".into()),
            show_deleted: Some(true),
        })
        .await
        .unwrap();
    assert_eq!(result.data_stores[0].id.as_deref(), Some("players"));
    assert_eq!(result.next_page_token.as_deref(), Some("next"));
    let requests = server.requests();
    assert_eq!(requests[0].method, "GET");
    assert!(requests[0]
        .target
        .starts_with("/cloud/v2/universes/123/data-stores?"));
    let query = requests[0]
        .target
        .split_once('?')
        .map(|(_, q)| q)
        .unwrap_or_default();
    assert!(query.contains("maxPageSize=50"));
    assert!(query.contains("pageToken=cursor"));
    assert!(query.contains("showDeleted=true"));
}

#[tokio::test]
async fn entry_methods_use_typed_bodies_and_encoded_paths() {
    let server = TestServer::start(vec![
        Reply::json(200, r#"{"id":"user/42","value":{"coins":7}}"#),
        Reply::json(201, r#"{"id":"created","value":[1,2]}"#),
        Reply::empty(204),
        Reply::json(200, r#"{"id":"user/42","value":12}"#),
        Reply::json(
            200,
            r#"{"newSnapshotTaken":true,"latestSnapshotTime":"2026-11-22T00:00:00Z"}"#,
        ),
    ]);
    let client = client(&server).await;
    let got = client.get_data_store_entry(path()).await.unwrap();
    assert_eq!(got.value, Some(json!({"coins":7})));
    client
        .create_data_store_entry(
            "123".into(),
            "player data".into(),
            "created".into(),
            DataStoreEntry {
                value: Some(json!([1, 2])),
                users: Some(vec!["users/42".into()]),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    client.delete_data_store_entry(path()).await.unwrap();
    client.increment_data_store_entry(path(), 5).await.unwrap();
    let snapshot = client
        .snapshot_data_stores("123".into(), SnapshotDataStoresRequest {})
        .await
        .unwrap();
    assert_eq!(snapshot.new_snapshot_taken, Some(true));

    let requests = server.requests();
    assert_eq!(
        requests[0].target,
        "/cloud/v2/universes/123/data-stores/player%20data/entries/user%2F42"
    );
    assert_eq!(requests[1].method, "POST");
    assert!(requests[1]
        .target
        .split_once('?')
        .map(|(_, q)| q)
        .unwrap_or_default()
        .contains("id=created"));
    let created: serde_json::Value = serde_json::from_str(&requests[1].body_text()).unwrap();
    assert_eq!(created["value"], json!([1, 2]));
    assert_eq!(created["users"], json!(["users/42"]));
    assert!(created.get("path").is_none());
    assert_eq!(requests[2].method, "DELETE");
    assert_eq!(requests[3].method, "POST");
    assert!(requests[3].target.ends_with(":increment"));
    let increment: serde_json::Value = serde_json::from_str(&requests[3].body_text()).unwrap();
    assert_eq!(increment, json!({"amount":5}));
    assert_eq!(
        requests[4].target,
        "/cloud/v2/universes/123/data-stores:snapshot"
    );
    assert_eq!(requests[4].body_text(), "{}");
    assert!(requests.iter().all(|r| r.header("x-api-key") == Some(KEY)));
}

#[test]
fn explicit_commands_have_no_generic_request_arguments() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_rbxcloud"))
        .args(["datastore-v2", "--help"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let help = String::from_utf8(output.stdout).unwrap();
    for command in [
        "list-data-stores",
        "get-data-store-entry",
        "create-data-store-entry",
        "delete-data-store-entry",
        "increment-data-store-entry",
        "snapshot-data-stores",
    ] {
        assert!(help.contains(command));
    }
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_rbxcloud"))
        .args(["datastore-v2", "create-data-store-entry", "--help"])
        .output()
        .unwrap();
    let help = String::from_utf8(output.stdout).unwrap();
    for argument in ["--id", "--value-json", "--user"] {
        assert!(help.contains(argument));
    }
    for argument in ["--url", "--method", "--path", "--body", "--api-key"] {
        assert!(!help.contains(argument));
    }
}
