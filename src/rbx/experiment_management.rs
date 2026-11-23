//! Typed experiment create, update, and MDE calculation operations.
use super::{
    client::{AsyncClient, Error, Operation, Request, Server},
    config_publishing::ConditionalRuleDefinitionInput,
};
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

#[derive(Debug, Clone, Copy, clap::ValueEnum, Serialize, Deserialize)]
pub enum ExperimentProductType {
    #[serde(rename = "EXPERIMENT_PRODUCT_TYPE_IN_GAME_CONFIGS")]
    InGameConfigs,
    #[serde(rename = "EXPERIMENT_PRODUCT_TYPE_MATCHMAKING")]
    Matchmaking,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VariantMetaInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variant_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_baseline: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<i32>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExperimentConfigEntryInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entry_value: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SingleConfigExperimentVariantInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variant_meta: Option<VariantMetaInput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_entry: Option<ExperimentConfigEntryInput>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlaceScoringConfigInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub place_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matchmaking_scoring_config_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_platform_default: Option<bool>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MatchmakingExperimentVariantInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variant_meta: Option<VariantMetaInput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub place_matchmaking_configs: Option<Vec<PlaceScoringConfigInput>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InGameConfigExperimentConfigurationInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variants: Option<Vec<SingleConfigExperimentVariantInput>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MatchmakingExperimentConfigurationInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variants: Option<Vec<MatchmakingExperimentVariantInput>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExperimentConfigurationInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub product_type: Option<ExperimentProductType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub in_game_config_experiment_configuration: Option<InGameConfigExperimentConfigurationInput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matchmaking_experiment_configuration: Option<MatchmakingExperimentConfigurationInput>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TargetingCriteriaInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule: Option<ConditionalRuleDefinitionInput>,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExperimentInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub experiment_configuration: Option<ExperimentConfigurationInput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exposure_percent: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub targeting_criteria: Option<TargetingCriteriaInput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_seconds: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub universe_goal_metric: Option<GoalMetric>,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MdeInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exposure_percent: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_seconds: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub baseline_weight: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variant_weights: Option<Vec<f64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub universe_goal_metric: Option<GoalMetric>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub targeting_criteria: Option<TargetingCriteriaInput>,
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
