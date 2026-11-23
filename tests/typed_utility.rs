mod support;
use rbxcloud::rbx::{
    client::{AsyncClient, ClientPolicy},
    credentials::{Credentials, SecretString},
    generated::{utility::*, utility_models::*},
};
use std::time::Duration;
use support::{Reply, TestServer};

async fn client(server: &TestServer) -> UtilityCloudClient {
    let policy = ClientPolicy::loopback(server.addr())
        .unwrap()
        .timeout(Duration::from_secs(2))
        .retry_delay(Duration::from_millis(1));
    UtilityCloudClient::new(
        AsyncClient::new(
            Credentials::api_key(SecretString::new("utility-contract-test-key")),
            policy,
        )
        .await
        .unwrap(),
    )
}

#[tokio::test]
async fn utility_bodies_support_arrays_and_root_booleans() {
    let server = TestServer::start(
        (0..3)
            .map(|_| Reply::json(400, r#"{"message":"contract stop"}"#))
            .collect(),
    );
    let client = client(&server).await;
    let _ = client
        .generative_ai_translate_text(GenerativeAiTranslateTextParams {
            universe_id: "1".into(),
            body: TranslateTextRequestInput {
                text: "Hello".into(),
                source_language_code: Some("en-us".into()),
                target_language_codes: Some(vec!["fr-fr".into(), "ja-jp".into()]),
            },
        })
        .await;
    let _ = client
        .advertising_batch_get_campaign_status(AdvertisingBatchGetCampaignStatusParams {
            body: BatchGetStatusRequestInput {
                campaign_ids: vec!["a".into(), "b".into()],
            },
        })
        .await;
    let _ = client
        .game_localization_set_automatic_translation_status(
            GameLocalizationSetAutomaticTranslationStatusParams {
                game_id: 1,
                language_code: "fr".into(),
                body: true,
            },
        )
        .await;
    let requests = server.requests();
    assert_eq!(
        requests[0].body_text(),
        r#"{"text":"Hello","sourceLanguageCode":"en-us","targetLanguageCodes":["fr-fr","ja-jp"]}"#
    );
    assert_eq!(requests[1].body_text(), r#"{"campaignIds":["a","b"]}"#);
    assert_eq!(requests[2].body_text(), "true");
}

#[test]
fn root_boolean_body_is_an_explicit_flag() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_rbxcloud"))
        .args([
            "utility-tools",
            "game-localization",
            "set-automatic-translation-status",
            "--help",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let help = String::from_utf8(output.stdout).unwrap();
    assert!(help.contains("--body-value"));
}
