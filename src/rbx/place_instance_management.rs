//! Typed place data-model instance updates.

use super::client::{AsyncClient, Error, Operation, Request, Server};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Copy, clap::ValueEnum, Serialize, Deserialize)]
pub enum EngineRunContext {
    Legacy,
    Server,
    Client,
    Plugin,
}
impl std::fmt::Display for EngineRunContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct EngineScriptInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_context: Option<EngineRunContext>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct EngineModuleScriptInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct EngineFolderInput {}
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct EngineInstanceDetailsInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub folder: Option<EngineFolderInput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local_script: Option<EngineScriptInput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub module_script: Option<EngineModuleScriptInput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub script: Option<EngineScriptInput>,
}
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct EngineInstanceInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<EngineInstanceDetailsInput>,
}
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePlaceInstanceInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub engine_instance: Option<EngineInstanceInput>,
}
#[derive(Debug, Clone)]
pub struct PlaceInstanceClient {
    transport: AsyncClient,
}
impl PlaceInstanceClient {
    pub fn new(transport: AsyncClient) -> Self {
        Self { transport }
    }
    pub async fn update(
        &self,
        universe_id: u64,
        place_id: u64,
        instance_id: &str,
        input: UpdatePlaceInstanceInput,
        update_mask: Option<String>,
    ) -> Result<Value, Error> {
        let request = Request::new(Operation::patch(
            Server::Apis,
            "/cloud/v2/universes/{universe_id}/places/{place_id}/instances/{instance_id}",
        ))
        .path_param("universe_id", universe_id)
        .path_param("place_id", place_id)
        .path_param("instance_id", instance_id)
        .query_opt("updateMask", update_mask)
        .json(&input)?;
        self.transport.execute(request).await
    }
}
