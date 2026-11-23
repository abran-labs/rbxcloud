//! Typed Experience Config draft update, overwrite, and publish operations.

use super::client::{AsyncClient, Error, Operation, Request, Server};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

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

#[derive(Debug, Clone, Copy, clap::ValueEnum, Serialize, Deserialize)]
pub enum ConditionalRuleOperator {
    #[serde(rename = "OPERATOR_AND")]
    And,
    #[serde(rename = "OPERATOR_OR")]
    Or,
    #[serde(rename = "OPERATOR_NOT")]
    Not,
    #[serde(rename = "OPERATOR_EQ")]
    Eq,
    #[serde(rename = "OPERATOR_NE")]
    Ne,
    #[serde(rename = "OPERATOR_GT")]
    Gt,
    #[serde(rename = "OPERATOR_LT")]
    Lt,
    #[serde(rename = "OPERATOR_GTE")]
    Gte,
    #[serde(rename = "OPERATOR_LTE")]
    Lte,
    #[serde(rename = "OPERATOR_IN")]
    In,
    #[serde(rename = "OPERATOR_NIN")]
    Nin,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConditionalRuleLiteralValueInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub boolean_value: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub integer_value: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub double_value: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub string_value: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConditionalRuleOperandInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attribute_reference: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub literal_value: Option<ConditionalRuleLiteralValueInput>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConditionalRuleTokenInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operator: Option<ConditionalRuleOperator>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operand: Option<ConditionalRuleOperandInput>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConditionalRuleDefinitionInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokens: Option<Vec<ConditionalRuleTokenInput>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConditionalRulesInput {
    /// Null map values are rule tombstones on draft overwrite.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules: Option<BTreeMap<String, Option<ConditionalRuleDefinitionInput>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules_order: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DraftUpdateInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub draft_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entries: Option<BTreeMap<String, Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conditional_rules: Option<ConditionalRulesInput>,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DraftPublishInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub draft_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
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
