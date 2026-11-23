//! Typed localized game images, names, descriptions, and supported languages.

use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::client::{AsyncClient, Attachment, Error, Operation, Request, Server};

macro_rules! string_enum {
    ($name:ident { $($variant:ident),+ $(,)? }) => {
        #[derive(Debug, Clone, Copy, clap::ValueEnum, Serialize, Deserialize)]
        pub enum $name { $($variant),+ }
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{:?}", self) }
        }
    };
}

string_enum!(NameDescriptionUpdate {
    Invalid,
    Name,
    Description
});
string_enum!(LanguageCodeType { Language, Locale });

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NameDescriptionInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_type: Option<NameDescriptionUpdate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language_code: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PatchLanguageInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language_code_type: Option<LanguageCodeType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delete: Option<bool>,
}

#[derive(Serialize)]
struct NameDescriptionsBody {
    data: Vec<NameDescriptionInput>,
}

#[derive(Debug, Clone)]
pub struct GameLocalizationClient {
    transport: AsyncClient,
}

impl GameLocalizationClient {
    pub fn new(transport: AsyncClient) -> Self {
        Self { transport }
    }

    pub async fn upload_badge_icon(
        &self,
        id: u64,
        language: &str,
        file: &Path,
        content_type: &str,
    ) -> Result<Value, Error> {
        self.upload(Operation::post(Server::Apis, "/legacy-game-internationalization/v1/badges/{badgeId}/icons/language-codes/{languageCode}"), &[("badgeId", id.to_string()), ("languageCode", language.into())], file, content_type).await
    }

    pub async fn upload_developer_product_icon(
        &self,
        id: u64,
        language: &str,
        file: &Path,
        content_type: &str,
    ) -> Result<Value, Error> {
        self.upload(Operation::post(Server::Apis, "/legacy-game-internationalization/v1/developer-products/{developerProductId}/icons/language-codes/{languageCode}"), &[("developerProductId", id.to_string()), ("languageCode", language.into())], file, content_type).await
    }

    pub async fn upload_game_icon(
        &self,
        id: u64,
        language: &str,
        file: &Path,
        content_type: &str,
    ) -> Result<Value, Error> {
        self.upload(Operation::post(Server::Apis, "/legacy-game-internationalization/v1/game-icon/games/{gameId}/language-codes/{languageCode}"), &[("gameId", id.to_string()), ("languageCode", language.into())], file, content_type).await
    }

    pub async fn upload_game_pass_icon(
        &self,
        id: u64,
        language: &str,
        file: &Path,
        content_type: &str,
    ) -> Result<Value, Error> {
        self.upload(Operation::post(Server::Apis, "/legacy-game-internationalization/v1/game-passes/{gamePassId}/icons/language-codes/{languageCode}"), &[("gamePassId", id.to_string()), ("languageCode", language.into())], file, content_type).await
    }

    pub async fn upload_game_thumbnail(
        &self,
        id: u64,
        language: &str,
        file: &Path,
        content_type: &str,
    ) -> Result<Value, Error> {
        self.upload(Operation::post(Server::Apis, "/legacy-game-internationalization/v1/game-thumbnails/games/{gameId}/language-codes/{languageCode}/image"), &[("gameId", id.to_string()), ("languageCode", language.into())], file, content_type).await
    }

    pub async fn update_name_descriptions(
        &self,
        game_id: u64,
        data: Vec<NameDescriptionInput>,
    ) -> Result<Value, Error> {
        let request = Request::new(Operation::patch(
            Server::Apis,
            "/legacy-game-internationalization/v1/name-description/games/{gameId}",
        ))
        .path_param("gameId", game_id)
        .json(&NameDescriptionsBody { data })?;
        self.transport.execute(request).await
    }

    pub async fn update_supported_languages(
        &self,
        game_id: u64,
        languages: Vec<PatchLanguageInput>,
    ) -> Result<Value, Error> {
        let request = Request::new(Operation::patch(
            Server::Apis,
            "/legacy-game-internationalization/v1/supported-languages/games/{gameId}",
        ))
        .path_param("gameId", game_id)
        .json(&languages)?;
        self.transport.execute(request).await
    }

    async fn upload(
        &self,
        operation: Operation,
        params: &[(&'static str, String)],
        file: &Path,
        content_type: &str,
    ) -> Result<Value, Error> {
        let attachment = Attachment::from_path(file, content_type)?;
        let mut request = Request::new(operation);
        for (name, value) in params {
            request = request.path_param(*name, value);
        }
        self.transport
            .execute(request.multipart_file("Files", attachment))
            .await
    }
}
