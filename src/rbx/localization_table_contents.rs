//! Typed localization table content updates and translation-history queries.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::client::{AsyncClient, Error, Operation, Request, Server};

macro_rules! string_enum {
    ($name:ident { $($variant:ident),+ $(,)? }) => {
        #[derive(Debug, Clone, Copy, clap::ValueEnum, Serialize, Deserialize)]
        pub enum $name { $($variant),+ }
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{:?}", self) }
        }
    };
}

string_enum!(EntryFormat {
    Invalid,
    Legacy,
    Icu
});
string_enum!(ChangeAgentType {
    Invalid,
    User,
    Automation,
    Default
});
string_enum!(HistorySortOrder { Asc, Desc });
string_enum!(TranslationSourceType {
    Invalid,
    Text,
    AssetImage
});

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EntryIdentifierInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entry_format: Option<EntryFormat>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GameLocationInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EntryMetadataInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub example: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub game_locations: Option<Vec<GameLocationInput>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entry_format: Option<EntryFormat>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct ChangeAgentInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub change_agent_type: Option<ChangeAgentType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PatchTranslationInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub translation_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delete: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub change_agent: Option<ChangeAgentInput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_time: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PatchEntryInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identifier: Option<EntryIdentifierInput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<EntryMetadataInput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub translations: Option<Vec<PatchTranslationInput>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delete: Option<bool>,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTableContentsInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub entries: Vec<PatchEntryInput>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CursorEntryIdentifierInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identifier: Option<EntryIdentifierInput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_order: Option<HistorySortOrder>,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TranslationHistoryInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,
    pub entries: Vec<CursorEntryIdentifierInput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_type: Option<TranslationSourceType>,
}

#[derive(Debug, Clone)]
pub struct LocalizationTableContentClient {
    transport: AsyncClient,
}

impl LocalizationTableContentClient {
    pub fn new(transport: AsyncClient) -> Self {
        Self { transport }
    }

    pub async fn update_table(
        &self,
        table_id: String,
        input: UpdateTableContentsInput,
    ) -> Result<Value, Error> {
        let request = Request::new(Operation::patch(
            Server::Apis,
            "/legacy-localization-tables/v1/localization-table/tables/{tableId}",
        ))
        .path_param("tableId", table_id)
        .json(&input)?;
        self.transport.execute(request).await
    }

    pub async fn translation_history(
        &self,
        table_id: String,
        input: TranslationHistoryInput,
    ) -> Result<Value, Error> {
        let request = Request::new(Operation::post(Server::Apis, "/legacy-localization-tables/v1/localization-table/tables/{tableId}/entries/translation-history"))
            .path_param("tableId", table_id)
            .json(&input)?;
        self.transport.execute(request).await
    }
}
