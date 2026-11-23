//! Typed experiment create, update, and MDE calculation operations.
use super::client::{AsyncClient, Error, Operation, Request, Server};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Copy, clap::ValueEnum, Serialize, Deserialize)]
pub enum GoalMetric {
    #[serde(rename = "UNIVERSE_EXPERIMENT_METRIC_AVERAGE_SESSION_TIME")]
    AverageSessionTime,
    #[serde(rename = "UNIVERSE_EXPERIMENT_METRIC_PLAYTIME_PER_USER")]
    PlaytimePerUser,
    #[serde(rename = "UNIVERSE_EXPERIMENT_METRIC_DAY_1_RETENTION")]
    Day1Retention,
    #[serde(rename = "UNIVERSE_EXPERIMENT_METRIC_DAY_7_RETENTION")]
    Day7Retention,
    #[serde(rename = "UNIVERSE_EXPERIMENT_METRIC_PAYER_CONVERSION_RATE")]
    PayerConversionRate,
    #[serde(rename = "UNIVERSE_EXPERIMENT_METRIC_AVERAGE_REVENUE_PER_USER")]
    AverageRevenuePerUser,
    #[serde(rename = "UNIVERSE_EXPERIMENT_METRIC_AVERAGE_REVENUE_PER_PAYING_USER")]
    AverageRevenuePerPayingUser,
}
impl std::fmt::Display for GoalMetric {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::AverageSessionTime => "UNIVERSE_EXPERIMENT_METRIC_AVERAGE_SESSION_TIME",
            Self::PlaytimePerUser => "UNIVERSE_EXPERIMENT_METRIC_PLAYTIME_PER_USER",
            Self::Day1Retention => "UNIVERSE_EXPERIMENT_METRIC_DAY_1_RETENTION",
            Self::Day7Retention => "UNIVERSE_EXPERIMENT_METRIC_DAY_7_RETENTION",
            Self::PayerConversionRate => "UNIVERSE_EXPERIMENT_METRIC_PAYER_CONVERSION_RATE",
            Self::AverageRevenuePerUser => "UNIVERSE_EXPERIMENT_METRIC_AVERAGE_REVENUE_PER_USER",
            Self::AverageRevenuePerPayingUser => {
                "UNIVERSE_EXPERIMENT_METRIC_AVERAGE_REVENUE_PER_PAYING_USER"
            }
        })
    }
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExperimentInput {
    pub name: Option<String>,
    pub description: Option<String>,
    pub experiment_configuration: Option<Value>,
    pub exposure_percent: Option<u8>,
    pub targeting_criteria: Option<Value>,
    pub duration_seconds: Option<u64>,
    pub universe_goal_metric: Option<GoalMetric>,
}
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MdeInput {
    pub exposure_percent: Option<u8>,
    pub duration_seconds: Option<u64>,
    pub baseline_weight: Option<f64>,
    pub variant_weights: Option<Vec<f64>>,
    pub universe_goal_metric: Option<GoalMetric>,
    pub targeting_criteria: Option<Value>,
}

#[derive(Debug, Clone)]
pub struct ExperimentClient {
    transport: AsyncClient,
}
impl ExperimentClient {
    pub fn new(transport: AsyncClient) -> Self {
        Self { transport }
    }
    pub async fn create(&self, universe_id: u64, input: ExperimentInput) -> Result<Value, Error> {
        self.call(
            Operation::post(
                Server::Apis,
                "/creator-configs-public-api/v1/experimentation/universes/{universeId}/experiments",
            ),
            universe_id,
            None,
            &input,
        )
        .await
    }
    pub async fn update(
        &self,
        universe_id: u64,
        experiment_id: String,
        input: ExperimentInput,
    ) -> Result<Value, Error> {
        self.call(Operation::patch(Server::Apis, "/creator-configs-public-api/v1/experimentation/universes/{universeId}/experiments/{experimentId}"), universe_id, Some(experiment_id), &input).await
    }
    pub async fn calculate_mde(&self, universe_id: u64, input: MdeInput) -> Result<Value, Error> {
        self.call(Operation::post(Server::Apis, "/creator-configs-public-api/v1/experimentation/universes/{universeId}/experiments:calculateMde"), universe_id, None, &input).await
    }
    async fn call(
        &self,
        operation: Operation,
        universe_id: u64,
        experiment_id: Option<String>,
        body: &impl Serialize,
    ) -> Result<Value, Error> {
        let mut request = Request::new(operation).path_param("universeId", universe_id);
        if let Some(id) = experiment_id {
            request = request.path_param("experimentId", id);
        }
        self.transport.execute(request.json(body)?).await
    }
}
