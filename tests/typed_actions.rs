mod support;
use rbxcloud::rbx::{
    client::{AsyncClient, ClientPolicy},
    credentials::{Credentials, SecretString},
    generated::{actions::*, actions_models::*},
};
use std::time::Duration;
use support::{Reply, TestServer};

async fn client(server: &TestServer) -> ActionsCloudClient {
    let policy = ClientPolicy::loopback(server.addr())
        .unwrap()
        .timeout(Duration::from_secs(2))
        .retry_delay(Duration::from_millis(1));
    ActionsCloudClient::new(
        AsyncClient::new(
            Credentials::api_key(SecretString::new("actions-contract-test-key")),
            policy,
        )
        .await
        .unwrap(),
    )
}

#[tokio::test]
async fn action_bodies_preserve_exact_json_fields() {
    let server = TestServer::start(
        (0..4)
            .map(|_| Reply::json(400, r#"{"message":"contract stop"}"#))
            .collect(),
    );
    let client = client(&server).await;
    let _ = client
        .advertising_create_creative(AdvertisingCreateCreativeParams {
            group_id: Some("7".into()),
            body: CreateCreativeRequestInput {
                asset_id: "123".into(),
                universe_id: Some("9".into()),
            },
        })
        .await;
    let _ = client
        .item_folders_create(ItemFoldersCreateParams {
            body: RobloxItemConfigurationApiModelsRequestFolderCreateFolderRequestInput {
                name: Some("Icons".into()),
                parent_folder_id: Some("p".into()),
                group_id: Some(7),
            },
        })
        .await;
    let _ = client
        .game_localization_update_badge_name(GameLocalizationUpdateBadgeNameParams {
            badge_id: 1,
            language_code: "fr".into(),
            body: RobloxGameInternationalizationApiUpdateBadgeNameRequestInput {
                name: Some("Nom".into()),
            },
        })
        .await;
    let _ = client
        .secrets_create(SecretsCreateParams {
            universe_id: 1,
            body: SecretInput {
                id: Some("api".into()),
                secret: Some("encrypted".into()),
                key_id: Some("key".into()),
                domain: Some("example.com".into()),
                ..Default::default()
            },
        })
        .await;
    let requests = server.requests();
    assert_eq!(requests[0].target, "/ads-management/v1/creatives?groupId=7");
    assert_eq!(
        requests[0].body_text(),
        r#"{"assetId":"123","universeId":"9"}"#
    );
    assert_eq!(
        requests[1].body_text(),
        r#"{"name":"Icons","parentFolderId":"p","groupId":7}"#
    );
    assert_eq!(
        requests[2].target,
        "/legacy-game-internationalization/v1/badges/1/name/language-codes/fr"
    );
    assert_eq!(requests[2].body_text(), r#"{"name":"Nom"}"#);
    assert_eq!(
        requests[3].body_text(),
        r#"{"id":"api","secret":"encrypted","key_id":"key","domain":"example.com"}"#
    );
}

#[test]
fn action_cli_uses_explicit_family_subgroups() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_rbxcloud"))
        .args(["action-tools", "--help"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let help = String::from_utf8(output.stdout).unwrap();
    for group in [
        "advertising",
        "game-localization",
        "item-folders",
        "secrets",
    ] {
        assert!(help.contains(group));
    }
}
