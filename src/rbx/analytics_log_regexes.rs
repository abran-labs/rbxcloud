//! Typed analytics log attribute regex configuration operations.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::client::{AsyncClient, Error, Operation, Request, Server};

#[derive(Debug, Clone, Copy, clap::ValueEnum, Serialize, Deserialize)]
pub enum RegexOperation {
    Invalid,
    Ignore,
    Group,
}
impl std::fmt::Display for RegexOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateLogRegexInput {
    pub universe_id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regex_operation: Option<RegexOperation>,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateLogRegexInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regex_operation: Option<RegexOperation>,
}

#[derive(Debug, Clone)]
pub struct AnalyticsLogRegexClient {
    transport: AsyncClient,
}

impl AnalyticsLogRegexClient {
    pub fn new(transport: AsyncClient) -> Self {
        Self { transport }
    }

    pub async fn create_regex(&self, input: CreateLogRegexInput) -> Result<Value, Error> {
        let request = Request::new(Operation::post(
            Server::Apis,
            "/log-attribute-http-service/v1/universe-regexes",
        ))
        .json(&input)?;
        self.transport.execute(request).await
    }

    pub async fn update_regex(&self, id: &str, input: UpdateLogRegexInput) -> Result<Value, Error> {
        let request = Request::new(Operation::put(
            Server::Apis,
            "/log-attribute-http-service/v1/universe-regexes/{id}",
        ))
        .path_param("id", id)
        .json(&input)?;
        self.transport.execute(request).await
    }
}
