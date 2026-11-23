mod support;

use rbxcloud::rbx::{
    client::{AsyncClient, ClientPolicy},
    credentials::{Credentials, SecretString},
    generated::{models::*, storage::*},
};
use serde_json::json;
use std::time::Duration;
use support::{Reply, TestServer};

async fn client(server: &TestServer) -> CloudClient {
    let policy = ClientPolicy::loopback(server.addr())
        .unwrap()
        .timeout(Duration::from_secs(2))
        .retry_delay(Duration::from_millis(1));
    CloudClient::new(
        AsyncClient::new(
            Credentials::api_key(SecretString::new("storage-contract-test-key")),
            policy,
        )
        .await
        .unwrap(),
    )
}

fn replies(count: usize) -> Vec<Reply> {
    (0..count).map(|_| Reply::json(200, "{}")).collect()
}

#[tokio::test]
async fn all_eighteen_data_store_methods_hit_the_generated_client() {
    let server = TestServer::start(replies(18));
    let client = client(&server).await;
    client
        .list_data_stores(ListDataStoresParams {
            universe_id: "u".into(),
            max_page_size: Some(10),
            page_token: Some("page".into()),
            filter: Some("id.startsWith(\"p\")".into()),
            show_deleted: Some(true),
        })
        .await
        .unwrap();
    client
        .delete_data_store(DeleteDataStoreParams {
            universe_id: "u".into(),
            data_store_id: "d".into(),
        })
        .await
        .unwrap();
    client
        .list_data_store_entries(ListDataStoreEntriesParams {
            universe_id: "u".into(),
            data_store_id: "d".into(),
            max_page_size: Some(20),
            page_token: None,
            filter: None,
            show_deleted: None,
        })
        .await
        .unwrap();
    client
        .create_data_store_entry(CreateDataStoreEntryParams {
            universe_id: "u".into(),
            data_store_id: "d".into(),
            id: Some("entry".into()),
            body: DataStoreEntryInput {
                value: Some(json!({"nested":[1,true,"x"]})),
                attributes: Some(json!({"tag":"a"})),
                ..Default::default()
            },
        })
        .await
        .unwrap();
    client
        .get_data_store_entry(GetDataStoreEntryParams {
            universe_id: "u".into(),
            data_store_id: "d".into(),
            entry_id: "e".into(),
        })
        .await
        .unwrap();
    client
        .update_data_store_entry(UpdateDataStoreEntryParams {
            universe_id: "u".into(),
            data_store_id: "d".into(),
            entry_id: "e".into(),
            allow_missing: Some(true),
            body: DataStoreEntryInput {
                value: Some(json!(null)),
                ..Default::default()
            },
        })
        .await
        .unwrap();
    client
        .delete_data_store_entry(DeleteDataStoreEntryParams {
            universe_id: "u".into(),
            data_store_id: "d".into(),
            entry_id: "e".into(),
        })
        .await
        .unwrap();
    client
        .increment_data_store_entry(IncrementDataStoreEntryParams {
            universe_id: "u".into(),
            data_store_id: "d".into(),
            entry_id: "e".into(),
            body: IncrementDataStoreEntryRequestInput {
                amount: Some(5),
                users: None,
                attributes: None,
            },
        })
        .await
        .unwrap();
    client
        .list_data_store_entry_revisions(ListDataStoreEntryRevisionsParams {
            universe_id: "u".into(),
            data_store_id: "d".into(),
            entry_id: "e".into(),
            max_page_size: Some(2),
            page_token: None,
            filter: None,
        })
        .await
        .unwrap();
    client
        .list_scoped_data_store_entries(ListScopedDataStoreEntriesParams {
            universe_id: "u".into(),
            data_store_id: "d".into(),
            scope_id: "s".into(),
            max_page_size: None,
            page_token: None,
            filter: None,
            show_deleted: None,
        })
        .await
        .unwrap();
    client
        .create_scoped_data_store_entry(CreateScopedDataStoreEntryParams {
            universe_id: "u".into(),
            data_store_id: "d".into(),
            scope_id: "s".into(),
            id: Some("e".into()),
            body: DataStoreEntryInput {
                value: Some(json!(["a", 2])),
                ..Default::default()
            },
        })
        .await
        .unwrap();
    client
        .get_scoped_data_store_entry(GetScopedDataStoreEntryParams {
            universe_id: "u".into(),
            data_store_id: "d".into(),
            scope_id: "s".into(),
            entry_id: "e".into(),
        })
        .await
        .unwrap();
    client
        .update_scoped_data_store_entry(UpdateScopedDataStoreEntryParams {
            universe_id: "u".into(),
            data_store_id: "d".into(),
            scope_id: "s".into(),
            entry_id: "e".into(),
            allow_missing: None,
            body: DataStoreEntryInput {
                value: Some(json!("quoted string")),
                ..Default::default()
            },
        })
        .await
        .unwrap();
    client
        .delete_scoped_data_store_entry(DeleteScopedDataStoreEntryParams {
            universe_id: "u".into(),
            data_store_id: "d".into(),
            scope_id: "s".into(),
            entry_id: "e".into(),
        })
        .await
        .unwrap();
    client
        .increment_scoped_data_store_entry(IncrementScopedDataStoreEntryParams {
            universe_id: "u".into(),
            data_store_id: "d".into(),
            scope_id: "s".into(),
            entry_id: "e".into(),
            body: IncrementDataStoreEntryRequestInput {
                amount: Some(-3),
                users: None,
                attributes: None,
            },
        })
        .await
        .unwrap();
    client
        .list_scoped_data_store_entry_revisions(ListScopedDataStoreEntryRevisionsParams {
            universe_id: "u".into(),
            data_store_id: "d".into(),
            scope_id: "s".into(),
            entry_id: "e".into(),
            max_page_size: None,
            page_token: None,
            filter: None,
        })
        .await
        .unwrap();
    client
        .undelete_data_store(UndeleteDataStoreParams {
            universe_id: "u".into(),
            data_store_id: "d".into(),
            body: UndeleteDataStoreRequestInput {},
        })
        .await
        .unwrap();
    client
        .snapshot_data_stores(SnapshotDataStoresParams {
            universe_id: "u".into(),
            body: SnapshotDataStoresRequestInput {},
        })
        .await
        .unwrap();

    let requests = server.requests();
    assert_eq!(requests.len(), 18);
    let expected = [
        ("GET", "/cloud/v2/universes/u/data-stores?maxPageSize=10&pageToken=page&filter=id.startsWith%28%22p%22%29&showDeleted=true"),
        ("DELETE", "/cloud/v2/universes/u/data-stores/d"),
        ("GET", "/cloud/v2/universes/u/data-stores/d/entries?maxPageSize=20"),
        ("POST", "/cloud/v2/universes/u/data-stores/d/entries?id=entry"),
        ("GET", "/cloud/v2/universes/u/data-stores/d/entries/e"),
        ("PATCH", "/cloud/v2/universes/u/data-stores/d/entries/e?allowMissing=true"),
        ("DELETE", "/cloud/v2/universes/u/data-stores/d/entries/e"),
        ("POST", "/cloud/v2/universes/u/data-stores/d/entries/e:increment"),
        ("GET", "/cloud/v2/universes/u/data-stores/d/entries/e:listRevisions?maxPageSize=2"),
        ("GET", "/cloud/v2/universes/u/data-stores/d/scopes/s/entries"),
        ("POST", "/cloud/v2/universes/u/data-stores/d/scopes/s/entries?id=e"),
        ("GET", "/cloud/v2/universes/u/data-stores/d/scopes/s/entries/e"),
        ("PATCH", "/cloud/v2/universes/u/data-stores/d/scopes/s/entries/e"),
        ("DELETE", "/cloud/v2/universes/u/data-stores/d/scopes/s/entries/e"),
        ("POST", "/cloud/v2/universes/u/data-stores/d/scopes/s/entries/e:increment"),
        ("GET", "/cloud/v2/universes/u/data-stores/d/scopes/s/entries/e:listRevisions"),
        ("POST", "/cloud/v2/universes/u/data-stores/d:undelete"),
        ("POST", "/cloud/v2/universes/u/data-stores:snapshot"),
    ];
    for (request, (method, target)) in requests.iter().zip(expected) {
        assert_eq!(request.method, method);
        assert_eq!(request.target, target);
    }
    assert_eq!(
        json_body(&requests[3]),
        json!({"value":{"nested":[1,true,"x"]},"attributes":{"tag":"a"}})
    );
    assert_eq!(json_body(&requests[5]), json!({"value":null}));
    assert_eq!(json_body(&requests[7]), json!({"amount":5}));
    assert_eq!(json_body(&requests[10]), json!({"value":["a",2]}));
    assert_eq!(json_body(&requests[12]), json!({"value":"quoted string"}));
    assert_eq!(json_body(&requests[14]), json!({"amount":-3}));
    assert_eq!(requests[16].body_text(), "{}");
    assert_eq!(requests[17].body_text(), "{}");
}

#[tokio::test]
async fn all_ten_memory_store_methods_hit_the_generated_client() {
    let server = TestServer::start(replies(10));
    let client = client(&server).await;
    client
        .get_memory_store_flush_operation(GetMemoryStoreFlushOperationParams {
            universe_id: 1,
            operation_id: "o".into(),
            scope: Some(GetMemoryStoreFlushOperationScope::Live),
        })
        .await
        .unwrap();
    client
        .create_memory_store_queue_item(CreateMemoryStoreQueueItemParams {
            universe_id: 1,
            queue_id: "q".into(),
            body: CreateMemoryStoreQueueItemRequestInput {
                data: Some(json!({"x":1})),
                ttl: Some("3s".into()),
                priority: Some(2.5),
            },
        })
        .await
        .unwrap();
    client
        .discard_memory_store_queue_items(DiscardMemoryStoreQueueItemsParams {
            universe_id: 1,
            queue_id: "q".into(),
            body: DiscardMemoryStoreQueueItemsRequestInput {
                read_id: Some("r".into()),
            },
        })
        .await
        .unwrap();
    client
        .read_memory_store_queue_items(ReadMemoryStoreQueueItemsParams {
            universe_id: 1,
            queue_id: "q".into(),
            count: Some(2),
            all_or_nothing: Some(true),
            invisibility_window: Some("3s".into()),
        })
        .await
        .unwrap();
    client
        .list_memory_store_sorted_map_items(ListMemoryStoreSortedMapItemsParams {
            universe_id: 1,
            sorted_map_id: "m".into(),
            max_page_size: Some(4),
            page_token: None,
            order_by: Some("key".into()),
            filter: None,
        })
        .await
        .unwrap();
    client
        .create_memory_store_sorted_map_item(CreateMemoryStoreSortedMapItemParams {
            universe_id: 1,
            sorted_map_id: "m".into(),
            id: "i".into(),
            body: MemoryStoreSortedMapItemInput {
                value: Some(json!({"a":1})),
                ttl: Some("3s".into()),
                ..Default::default()
            },
        })
        .await
        .unwrap();
    client
        .get_memory_store_sorted_map_item(GetMemoryStoreSortedMapItemParams {
            universe_id: 1,
            sorted_map_id: "m".into(),
            item_id: "i".into(),
        })
        .await
        .unwrap();
    client
        .update_memory_store_sorted_map_item(UpdateMemoryStoreSortedMapItemParams {
            universe_id: 1,
            sorted_map_id: "m".into(),
            item_id: "i".into(),
            allow_missing: Some(true),
            body: MemoryStoreSortedMapItemInput {
                value: Some(json!([1, 2])),
                ttl: Some("4s".into()),
                ..Default::default()
            },
        })
        .await
        .unwrap();
    client
        .delete_memory_store_sorted_map_item(DeleteMemoryStoreSortedMapItemParams {
            universe_id: 1,
            sorted_map_id: "m".into(),
            item_id: "i".into(),
        })
        .await
        .unwrap();
    client
        .flush_memory_store(FlushMemoryStoreParams {
            universe_id: 1,
            scope: Some(FlushMemoryStoreScope::Live),
        })
        .await
        .unwrap();
    let requests = server.requests();
    assert_eq!(requests.len(), 10);
    for (request, suffix) in requests.iter().zip([
        "operations/o?scope=LIVE",
        "queues/q/items",
        "queues/q/items:discard",
        "queues/q/items:read?count=2&allOrNothing=true&invisibilityWindow=3s",
        "sorted-maps/m/items?maxPageSize=4&orderBy=key",
        "sorted-maps/m/items?id=i",
        "sorted-maps/m/items/i",
        "sorted-maps/m/items/i?allowMissing=true",
        "sorted-maps/m/items/i",
        ":flush?scope=LIVE",
    ]) {
        assert!(
            request.target.ends_with(suffix),
            "{} does not end in {suffix}",
            request.target
        );
    }
    assert_eq!(
        json_body(&requests[1]),
        json!({"data":{"x":1},"ttl":"3s","priority":2.5})
    );
    assert_eq!(json_body(&requests[2]), json!({"readId":"r"}));
    assert_eq!(json_body(&requests[5]), json!({"value":{"a":1},"ttl":"3s"}));
    assert_eq!(json_body(&requests[7]), json!({"value":[1,2],"ttl":"4s"}));
}

#[tokio::test]
async fn all_six_ordered_store_methods_hit_the_generated_client() {
    let server = TestServer::start(replies(6));
    let client = client(&server).await;
    client
        .list_ordered_data_store_entries(ListOrderedDataStoreEntriesParams {
            universe_id: "u".into(),
            ordered_data_store_id: "d".into(),
            scope_id: "s".into(),
            max_page_size: Some(5),
            page_token: None,
            order_by: None,
            filter: None,
        })
        .await
        .unwrap();
    client
        .create_ordered_data_store_entry(CreateOrderedDataStoreEntryParams {
            universe_id: "u".into(),
            ordered_data_store_id: "d".into(),
            scope_id: "s".into(),
            id: Some("e".into()),
            body: OrderedDataStoreEntryInput {
                value: Some(5),
                ..Default::default()
            },
        })
        .await
        .unwrap();
    client
        .get_ordered_data_store_entry(GetOrderedDataStoreEntryParams {
            universe_id: "u".into(),
            ordered_data_store_id: "d".into(),
            scope_id: "s".into(),
            entry_id: "e".into(),
        })
        .await
        .unwrap();
    client
        .update_ordered_data_store_entry(UpdateOrderedDataStoreEntryParams {
            universe_id: "u".into(),
            ordered_data_store_id: "d".into(),
            scope_id: "s".into(),
            entry_id: "e".into(),
            allow_missing: None,
            body: OrderedDataStoreEntryInput {
                value: Some(7),
                ..Default::default()
            },
        })
        .await
        .unwrap();
    client
        .delete_ordered_data_store_entry(DeleteOrderedDataStoreEntryParams {
            universe_id: "u".into(),
            ordered_data_store_id: "d".into(),
            scope_id: "s".into(),
            entry_id: "e".into(),
        })
        .await
        .unwrap();
    client
        .increment_ordered_data_store_entry(IncrementOrderedDataStoreEntryParams {
            universe_id: "u".into(),
            ordered_data_store_id: "d".into(),
            scope_id: "s".into(),
            entry_id: "e".into(),
            body: IncrementOrderedDataStoreEntryRequestInput { amount: Some(2) },
        })
        .await
        .unwrap();
    let requests = server.requests();
    assert_eq!(requests.len(), 6);
    assert!(requests[0].target.ends_with("/entries?maxPageSize=5"));
    assert_eq!(json_body(&requests[1]), json!({"value":5}));
    assert_eq!(json_body(&requests[3]), json!({"value":7}));
    assert_eq!(json_body(&requests[5]), json!({"amount":2}));
}

fn json_body(request: &support::Recorded) -> serde_json::Value {
    serde_json::from_str(&request.body_text()).unwrap()
}

#[test]
fn generated_commands_have_no_generic_request_arguments_or_read_only_fields() {
    for group in ["datastore-v2", "memory-store", "ordered-datastore-v2"] {
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_rbxcloud"))
            .args([group, "--help"])
            .output()
            .unwrap();
        assert!(output.status.success(), "{group}");
    }
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_rbxcloud"))
        .args(["datastore-v2", "create-data-store-entry", "--help"])
        .output()
        .unwrap();
    let help = String::from_utf8(output.stdout).unwrap();
    for argument in ["--id", "--body-value", "--body-users", "--body-attributes"] {
        assert!(help.contains(argument));
    }
    for argument in [
        "--url ",
        "--method ",
        "--path ",
        "--body ",
        "--api-key ",
        "--body-id ",
        "--body-create-time ",
        "--body-revision-id ",
    ] {
        assert!(!help.contains(argument), "unexpected {argument}");
    }
}

#[test]
fn parser_accepts_negative_numbers_and_validates_memory_scope() {
    let binary = env!("CARGO_BIN_EXE_rbxcloud");
    let valid = std::process::Command::new(binary)
        .args([
            "ordered-datastore-v2",
            "increment-ordered-data-store-entry",
            "--universe-id",
            "1",
            "--ordered-data-store-id",
            "d",
            "--scope-id",
            "s",
            "--entry-id",
            "e",
            "--body-amount",
            "-5",
        ])
        .env("ROBLOX_OPEN_CLOUD_API_KEY", "")
        .env("RBXCLOUD_API_KEY", "")
        .output()
        .unwrap();
    let stderr = String::from_utf8(valid.stderr).unwrap();
    assert!(!stderr.contains("unexpected argument '-5'"), "{stderr}");

    let invalid_scope = std::process::Command::new(binary)
        .args([
            "memory-store",
            "flush-memory-store",
            "--universe-id",
            "1",
            "--scope",
            "BOGUS",
        ])
        .output()
        .unwrap();
    assert_eq!(invalid_scope.status.code(), Some(2));
}

#[tokio::test]
async fn queue_read_does_not_retry_a_lost_or_rate_limited_response() {
    let server = TestServer::start(vec![Reply::json(503, r#"{"message":"unavailable"}"#)]);
    let result = client(&server)
        .await
        .read_memory_store_queue_items(ReadMemoryStoreQueueItemsParams {
            universe_id: 1,
            queue_id: "q".into(),
            count: Some(1),
            all_or_nothing: None,
            invisibility_window: None,
        })
        .await;
    assert!(result.is_err());
    assert_eq!(server.requests().len(), 1);
}

#[test]
fn response_enums_accept_future_states_but_request_enums_do_not_expose_unknown() {
    let store: DataStore = serde_json::from_value(json!({"state":"A_FUTURE_STATE"})).unwrap();
    assert_eq!(store.state, Some(DataStoreState::Unknown));
    let help = std::process::Command::new(env!("CARGO_BIN_EXE_rbxcloud"))
        .args(["memory-store", "flush-memory-store", "--help"])
        .output()
        .unwrap();
    let help = String::from_utf8(help.stdout).unwrap();
    assert!(help.contains("live"));
    assert!(help.contains("test"));
    assert!(!help.contains("unknown"));
}

#[test]
fn parser_accepts_negative_dynamic_json_values() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_rbxcloud"))
        .args([
            "datastore-v2",
            "create-data-store-entry",
            "--universe-id",
            "1",
            "--data-store-id",
            "d",
            "--body-value",
            "-5",
        ])
        .env("ROBLOX_OPEN_CLOUD_API_KEY", "")
        .env("RBXCLOUD_API_KEY", "")
        .output()
        .unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(!stderr.contains("unexpected argument '-5'"), "{stderr}");
}
