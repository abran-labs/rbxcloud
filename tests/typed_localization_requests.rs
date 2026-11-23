mod support;

use std::{path::PathBuf, time::Duration};

use rbxcloud::rbx::{
    client::{AsyncClient, ClientPolicy},
    credentials::{Credentials, SecretString},
    game_localization::*,
    localization_table_contents::*,
};
use support::{parse_multipart, Reply, TestServer};

async fn transport(server: &TestServer) -> AsyncClient {
    let policy = ClientPolicy::loopback(server.addr())
        .unwrap()
        .timeout(Duration::from_secs(2))
        .retry_delay(Duration::from_millis(1));
    AsyncClient::new(
        Credentials::api_key(SecretString::new("localization-request-test-key")),
        policy,
    )
    .await
    .unwrap()
}

fn image_file() -> PathBuf {
    let path =
        std::env::temp_dir().join(format!("rbxcloud-localization-{}.png", std::process::id()));
    std::fs::write(&path, b"png-bytes").unwrap();
    path
}

#[tokio::test]
async fn five_localized_image_operations_send_files_multipart_field() {
    let server = TestServer::start((0..5).map(|_| Reply::json(200, "{}")).collect());
    let client = GameLocalizationClient::new(transport(&server).await);
    let path = image_file();
    client
        .upload_badge_icon(1, "fr", &path, "image/png")
        .await
        .unwrap();
    client
        .upload_developer_product_icon(2, "de", &path, "image/png")
        .await
        .unwrap();
    client
        .upload_game_icon(3, "es", &path, "image/png")
        .await
        .unwrap();
    client
        .upload_game_pass_icon(4, "it", &path, "image/png")
        .await
        .unwrap();
    client
        .upload_game_thumbnail(5, "pt-br", &path, "image/png")
        .await
        .unwrap();
    std::fs::remove_file(path).unwrap();

    let requests = server.requests();
    let targets = [
        "/legacy-game-internationalization/v1/badges/1/icons/language-codes/fr",
        "/legacy-game-internationalization/v1/developer-products/2/icons/language-codes/de",
        "/legacy-game-internationalization/v1/game-icon/games/3/language-codes/es",
        "/legacy-game-internationalization/v1/game-passes/4/icons/language-codes/it",
        "/legacy-game-internationalization/v1/game-thumbnails/games/5/language-codes/pt-br/image",
    ];
    for (request, target) in requests.iter().zip(targets) {
        assert_eq!(request.target, target);
        let parts = parse_multipart(request.header("content-type").unwrap(), &request.body);
        assert_eq!(parts.len(), 1);
        assert_eq!(parts[0].name, "Files");
        assert_eq!(parts[0].content_type.as_deref(), Some("image/png"));
        assert_eq!(parts[0].data, b"png-bytes");
    }
}

#[tokio::test]
async fn name_description_and_supported_language_bodies_use_typed_arrays() {
    let server = TestServer::start(vec![Reply::json(200, "{}"), Reply::json(200, "{}")]);
    let client = GameLocalizationClient::new(transport(&server).await);
    client
        .update_name_descriptions(
            7,
            vec![
                NameDescriptionInput {
                    name: Some("Nom".into()),
                    description: None,
                    update_type: Some(NameDescriptionUpdate::Name),
                    language_code: Some("fr".into()),
                },
                NameDescriptionInput {
                    name: None,
                    description: Some("Description".into()),
                    update_type: Some(NameDescriptionUpdate::Description),
                    language_code: Some("en".into()),
                },
            ],
        )
        .await
        .unwrap();
    client
        .update_supported_languages(
            7,
            vec![PatchLanguageInput {
                language_code_type: Some(LanguageCodeType::Locale),
                language_code: Some("fr-ca".into()),
                delete: Some(false),
            }],
        )
        .await
        .unwrap();

    let requests = server.requests();
    let names: serde_json::Value = serde_json::from_str(&requests[0].body_text()).unwrap();
    assert_eq!(names["data"][0]["updateType"], "Name");
    assert_eq!(names["data"][1]["description"], "Description");
    assert_eq!(
        requests[1].target,
        "/legacy-game-internationalization/v1/supported-languages/games/7"
    );
    assert_eq!(
        requests[1].body_text(),
        r#"[{"languageCodeType":"Locale","languageCode":"fr-ca","delete":false}]"#
    );
}

#[tokio::test]
async fn table_update_serializes_nested_identifiers_metadata_and_translations() {
    let server = TestServer::start(vec![Reply::json(200, "{}")]);
    let client = LocalizationTableContentClient::new(transport(&server).await);
    client
        .update_table(
            "table".into(),
            UpdateTableContentsInput {
                name: Some("Strings".into()),
                entries: vec![PatchEntryInput {
                    identifier: Some(EntryIdentifierInput {
                        key: Some("Greeting".into()),
                        context: Some("Menu".into()),
                        source: Some("Hello".into()),
                        entry_format: Some(EntryFormat::Icu),
                    }),
                    metadata: Some(EntryMetadataInput {
                        example: Some("Hello, {player}".into()),
                        game_locations: Some(vec![GameLocationInput {
                            path: Some("Workspace.Sign".into()),
                        }]),
                        entry_format: Some(EntryFormat::Icu),
                    }),
                    translations: Some(vec![PatchTranslationInput {
                        locale: Some("fr".into()),
                        translation_text: Some("Bonjour".into()),
                        delete: Some(false),
                        change_agent: Some(ChangeAgentInput {
                            change_agent_type: Some(ChangeAgentType::User),
                            id: Some("42".into()),
                        }),
                        updated_time: Some("2026-01-01T00:00:00Z".into()),
                    }]),
                    delete: Some(false),
                }],
            },
        )
        .await
        .unwrap();

    let requests = server.requests();
    assert_eq!(
        requests[0].target,
        "/legacy-localization-tables/v1/localization-table/tables/table"
    );
    let body: serde_json::Value = serde_json::from_str(&requests[0].body_text()).unwrap();
    assert_eq!(body["entries"][0]["identifier"]["entryFormat"], "Icu");
    assert_eq!(
        body["entries"][0]["metadata"]["gameLocations"][0]["path"],
        "Workspace.Sign"
    );
    assert_eq!(
        body["entries"][0]["translations"][0]["changeAgent"]["ChangeAgentType"],
        "User"
    );
}

#[tokio::test]
async fn translation_history_serializes_cursors_counts_and_sort_order() {
    let server = TestServer::start(vec![Reply::json(200, "{}")]);
    let client = LocalizationTableContentClient::new(transport(&server).await);
    client
        .translation_history(
            "table".into(),
            TranslationHistoryInput {
                locale: Some("fr".into()),
                entries: vec![CursorEntryIdentifierInput {
                    cursor: Some("next".into()),
                    identifier: Some(EntryIdentifierInput {
                        key: Some("Greeting".into()),
                        ..Default::default()
                    }),
                    count: Some(20),
                    sort_order: Some(HistorySortOrder::Desc),
                }],
                source_type: Some(TranslationSourceType::Text),
            },
        )
        .await
        .unwrap();

    let requests = server.requests();
    assert_eq!(requests[0].target, "/legacy-localization-tables/v1/localization-table/tables/table/entries/translation-history");
    let body: serde_json::Value = serde_json::from_str(&requests[0].body_text()).unwrap();
    assert_eq!(body["entries"][0]["cursor"], "next");
    assert_eq!(body["entries"][0]["count"], 20);
    assert_eq!(body["entries"][0]["sortOrder"], "Desc");
    assert_eq!(body["sourceType"], "Text");
}
