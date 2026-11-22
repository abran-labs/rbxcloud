mod support;
use rbxcloud::rbx::{
    client::{AsyncClient, ClientPolicy},
    credentials::{Credentials, SecretString},
    generated::{
        asset_models::RollbackAssetVersionRequest,
        assets::{
            AssetCloudClient, GetAssetVersionParams, ListAssetQuotasParams,
            ListAssetVersionsParams, RollbackAssetVersionParams,
        },
    },
};
use std::time::Duration;
use support::{Reply, TestServer};

async fn client(server: &TestServer) -> AssetCloudClient {
    let policy = ClientPolicy::loopback(server.addr())
        .unwrap()
        .timeout(Duration::from_secs(2))
        .retry_delay(Duration::from_millis(1));
    AssetCloudClient::new(
        AsyncClient::new(
            Credentials::api_key(SecretString::new("asset-contract-test-key")),
            policy,
        )
        .await
        .unwrap(),
    )
}

#[tokio::test]
async fn asset_versions_and_quotas_use_documented_paths() {
    let server = TestServer::start(vec![
        Reply::json(
            200,
            r#"{"assetVersions":[{"path":"assets/12/versions/3","published":true}],"nextPageToken":"next"}"#,
        ),
        Reply::json(200, r#"{"path":"assets/12/versions/3","published":false}"#),
        Reply::json(200, r#"{"path":"assets/12/versions/2","published":true}"#),
        Reply::json(
            200,
            r#"{"assetQuotas":[{"path":"users/5/asset-quotas/image"}],"nextPageToken":null}"#,
        ),
    ]);
    let client = client(&server).await;
    let versions = client
        .list_asset_versions(ListAssetVersionsParams {
            asset_id: 12,
            max_page_size: Some(10),
            page_token: None,
            filter: None,
        })
        .await
        .unwrap();
    assert_eq!(
        versions.asset_versions.unwrap()[0].path.as_deref(),
        Some("assets/12/versions/3")
    );
    client
        .get_asset_version(GetAssetVersionParams {
            asset_id: 12,
            version_number: 3,
        })
        .await
        .unwrap();
    client
        .rollback_asset_version(RollbackAssetVersionParams {
            asset_id: 12,
            body: RollbackAssetVersionRequest {
                asset_version: "assets/12/versions/2".into(),
            },
        })
        .await
        .unwrap();
    client
        .list_asset_quotas(ListAssetQuotasParams {
            user_id: 5,
            max_page_size: None,
            page_token: None,
            filter: None,
        })
        .await
        .unwrap();
    let requests = server.requests();
    assert!(requests[0]
        .target
        .starts_with("/assets/v1/assets/12/versions?maxPageSize=10"));
    assert_eq!(requests[1].target, "/assets/v1/assets/12/versions/3");
    assert_eq!(requests[2].method, "POST");
    assert_eq!(requests[2].target, "/assets/v1/assets/12/versions:rollback");
    assert_eq!(
        requests[2].body_text(),
        r#"{"assetVersion":"assets/12/versions/2"}"#
    );
    assert_eq!(requests[3].target, "/cloud/v2/users/5/asset-quotas");
}

#[test]
fn legacy_assets_and_typed_asset_tools_are_both_present() {
    for args in [
        vec!["assets", "create", "--help"],
        vec!["asset-tools", "--help"],
    ] {
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_rbxcloud"))
            .args(args)
            .output()
            .unwrap();
        assert!(output.status.success());
    }
}
