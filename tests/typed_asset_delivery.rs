mod support;
use rbxcloud::rbx::{
    asset_delivery::*,
    client::{AsyncClient, ClientPolicy},
    credentials::{Credentials, SecretString},
};
use std::time::Duration;
use support::{Reply, TestServer};

async fn client(server: &TestServer) -> AssetDeliveryClient {
    let policy = ClientPolicy::loopback(server.addr())
        .unwrap()
        .timeout(Duration::from_secs(2))
        .retry_delay(Duration::from_millis(1));
    AssetDeliveryClient::new(
        AsyncClient::new(
            Credentials::api_key(SecretString::new("binary-download-test-key")),
            policy,
        )
        .await
        .unwrap(),
    )
}

#[tokio::test]
async fn binary_asset_download_preserves_bytes_and_version_path() {
    let server =
        TestServer::start(vec![Reply::binary(200, vec![0, 1, 2, 255, 10])
            .header("Content-Type", "application/octet-stream")]);
    let dir = tempfile::tempdir().unwrap();
    let output = dir.path().join("asset.bin");
    let result = client(&server)
        .await
        .download(
            AssetDownloadInput {
                asset_id: 5,
                version_number: Some(2),
                place_id: Some(9),
                ..Default::default()
            },
            &output,
        )
        .await
        .unwrap();
    assert_eq!(result.byte_length, 5);
    assert_eq!(
        result.content_type.as_deref(),
        Some("application/octet-stream")
    );
    assert_eq!(std::fs::read(&output).unwrap(), vec![0, 1, 2, 255, 10]);
    let requests = server.requests();
    assert_eq!(
        requests[0].target,
        "/asset-delivery-api/v1/assetId/5/version/2"
    );
    assert_eq!(requests[0].header("roblox-place-id"), Some("9"));
}
