mod support;
use rbxcloud::rbx::{
    asset_multipart::*,
    client::{AsyncClient, ClientPolicy},
    credentials::{Credentials, SecretString},
    generated::multipart_models::AssetsUploadApiCreator,
};
use std::time::Duration;
use support::{Reply, TestServer};

async fn client(server: &TestServer) -> AssetMultipartClient {
    let policy = ClientPolicy::loopback(server.addr())
        .unwrap()
        .timeout(Duration::from_secs(2))
        .retry_delay(Duration::from_millis(1));
    AssetMultipartClient::new(
        AsyncClient::new(
            Credentials::api_key(SecretString::new("multipart-start-test-key")),
            policy,
        )
        .await
        .unwrap(),
    )
}

fn input() -> MultipartStartInput {
    MultipartStartInput {
        asset: asset_metadata(
            "Icon".into(),
            "Game icon".into(),
            "Image".into(),
            AssetsUploadApiCreator {
                group_id: Some("7".into()),
                ..Default::default()
            },
            None,
        ),
        file: UploadFileInput {
            content_type: Some("image/png".into()),
            filesize: Some(100),
            md5_checksum: Some("abc".into()),
            chunk_plan: Some(vec![100]),
        },
    }
}

#[tokio::test]
async fn multipart_start_serializes_typed_asset_and_file_metadata() {
    let server = TestServer::start(vec![
        Reply::json(200, r#"{"operationPath":"operations/1"}"#),
        Reply::json(200, r#"{"operationPath":"operations/2"}"#),
    ]);
    let client = client(&server).await;
    client.create(input()).await.unwrap();
    client.update(9, input()).await.unwrap();
    let requests = server.requests();
    assert_eq!(requests[0].method, "POST");
    assert_eq!(requests[0].target, "/assets/v1/assets:multipartUpload");
    assert_eq!(requests[1].method, "PATCH");
    assert_eq!(requests[1].target, "/assets/v1/assets/9:multipartUpload");
    let body: serde_json::Value = serde_json::from_str(&requests[0].body_text()).unwrap();
    assert_eq!(body["asset"]["displayName"], "Icon");
    assert_eq!(body["asset"]["creationContext"]["creator"]["groupId"], "7");
    assert_eq!(body["file"]["contentType"], "image/png");
    assert_eq!(body["file"]["chunkPlan"], serde_json::json!([100]));
}
