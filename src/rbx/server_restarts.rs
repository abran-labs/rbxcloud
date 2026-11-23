//! Typed server restart launch requests.

use super::client::{AsyncClient, Error, Operation, Request, Server};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlaceVersionFilterInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub versions: Option<Vec<u32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude_current_version: Option<bool>,
}
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchRestartInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bleed_off_duration_minutes: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub places: Option<BTreeMap<String, PlaceVersionFilterInput>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attributes: Option<Value>,
}
#[derive(Debug, Clone)]
pub struct ServerRestartClient {
    transport: AsyncClient,
}
impl ServerRestartClient {
    pub fn new(transport: AsyncClient) -> Self {
        Self { transport }
    }
    pub async fn restart(
        &self,
        universe_id: u64,
        input: LaunchRestartInput,
    ) -> Result<Value, Error> {
        let request = Request::new(Operation::post(
            Server::Apis,
            "/server-management/v1/universes/{universeId}/restarts",
        ))
        .path_param("universeId", universe_id)
        .json(&input)?;
        self.transport.execute(request).await
    }
}
