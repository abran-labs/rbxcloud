//! Typed matchmaking player attribute definitions.
use super::client::{AsyncClient, Error, Operation, Request, Server};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, clap::ValueEnum, Serialize, Deserialize)]
pub enum AttributeDataType {
    Invalid,
    Bool,
    Number,
    String,
}
impl std::fmt::Display for AttributeDataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttributeValue {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bool_value: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub numeric_value: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub string_value: Option<String>,
    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_type: Option<AttributeDataType>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataStoreLocationInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_store_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_template: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_path: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttributeLocation {
    pub location_case: String,
    pub data_store_location: DataStoreLocationInput,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePlayerAttributeInput {
    pub universe_id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_type: Option<AttributeDataType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_value: Option<AttributeValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attribute_value_location: Option<AttributeLocation>,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePlayerAttributeInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attribute_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_value: Option<AttributeValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attribute_value_location: Option<AttributeLocation>,
}

#[derive(Debug, Clone)]
pub struct MatchmakingAttributeClient {
    transport: AsyncClient,
}
impl MatchmakingAttributeClient {
    pub fn new(transport: AsyncClient) -> Self {
        Self { transport }
    }
    pub async fn create(
        &self,
        input: CreatePlayerAttributeInput,
    ) -> Result<serde_json::Value, Error> {
        let request = Request::new(Operation::post(
            Server::Apis,
            "/matchmaking-api/v1/matchmaking/player-attribute",
        ))
        .json(&input)?;
        self.transport.execute(request).await
    }
    pub async fn update(
        &self,
        attribute_id: String,
        input: UpdatePlayerAttributeInput,
    ) -> Result<serde_json::Value, Error> {
        let request = Request::new(Operation::patch(
            Server::Apis,
            "/matchmaking-api/v1/matchmaking/player-attribute/{attributeId}",
        ))
        .path_param("attributeId", attribute_id)
        .json(&input)?;
        self.transport.execute(request).await
    }
}
