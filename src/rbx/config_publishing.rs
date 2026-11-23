//! Typed Experience Config draft update, overwrite, and publish operations.
use super::client::{AsyncClient, Error, Operation, Request, Server};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Copy, clap::ValueEnum, Serialize, Deserialize)]
pub enum DeploymentStrategy {
    Invalid,
    GradualRollout,
    Immediate,
}
impl std::fmt::Display for DeploymentStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConditionalRulesInput {
    pub rules: Option<Value>,
    pub rules_order: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DraftUpdateInput {
    pub draft_hash: Option<String>,
    pub entries: Option<Value>,
    pub conditional_rules: Option<ConditionalRulesInput>,
}
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DraftPublishInput {
    pub draft_hash: Option<String>,
    pub message: Option<String>,
    pub deployment_strategy: Option<DeploymentStrategy>,
}

#[derive(Debug, Clone)]
pub struct ConfigPublishingClient {
    transport: AsyncClient,
}
impl ConfigPublishingClient {
    pub fn new(transport: AsyncClient) -> Self {
        Self { transport }
    }
    pub async fn update(
        &self,
        universe_id: String,
        repository: String,
        input: DraftUpdateInput,
    ) -> Result<Value, Error> {
        self.call(Operation::patch(Server::Apis, "/creator-configs-public-api/v1/configs/universes/{universeId}/repositories/{repository}/draft"), universe_id, repository, &input).await
    }
    pub async fn overwrite(
        &self,
        universe_id: String,
        repository: String,
        input: DraftUpdateInput,
    ) -> Result<Value, Error> {
        self.call(Operation::put(Server::Apis, "/creator-configs-public-api/v1/configs/universes/{universeId}/repositories/{repository}/draft:overwrite"), universe_id, repository, &input).await
    }
    pub async fn publish(
        &self,
        universe_id: String,
        repository: String,
        input: DraftPublishInput,
    ) -> Result<Value, Error> {
        self.call(Operation::post(Server::Apis, "/creator-configs-public-api/v1/configs/universes/{universeId}/repositories/{repository}/publish"), universe_id, repository, &input).await
    }
    async fn call(
        &self,
        operation: Operation,
        universe_id: String,
        repository: String,
        body: &impl Serialize,
    ) -> Result<Value, Error> {
        let request = Request::new(operation)
            .path_param("universeId", universe_id)
            .path_param("repository", repository)
            .json(body)?;
        self.transport.execute(request).await
    }
}
