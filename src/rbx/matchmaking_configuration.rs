//! Typed matchmaking server attributes, scoring configurations, and custom signals.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{
    client::{AsyncClient, Error, Operation, Request, Server},
    matchmaking_attributes::{AttributeDataType, AttributeValue},
};

macro_rules! string_enum {
    ($name:ident { $($variant:ident),+ $(,)? }) => {
        #[derive(Debug, Clone, Copy, clap::ValueEnum, Serialize, Deserialize)]
        pub enum $name { $($variant),+ }
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{:?}", self)
            }
        }
    };
}

string_enum!(ServerDefaultSource {
    Invalid,
    Constant,
    PlayerAttributeReference
});
string_enum!(ScoringTemplate {
    NotApplicable,
    UseDefault
});
string_enum!(CustomSignalType {
    Invalid,
    PlayerCategorical,
    ServerCategorical,
    ServerNumerical,
    PlayerNumerical
});
string_enum!(SignalCurveType {
    Invalid,
    PositiveLinear,
    NegativeLinear
});
string_enum!(CategoricalComparison {
    Invalid,
    EqualityToPlayer,
    EqualityToConstant
});
string_enum!(NumericalComparison {
    Invalid,
    AbsoluteDifferentToPlayer,
    AbsoluteDifferentToConstant
});
string_enum!(AggregationFunction {
    Invalid,
    Average,
    Median,
    Sum
});

#[derive(Debug, Clone, Default, Serialize)]
pub struct AttributeReferenceInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerDefaultValueInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_case: Option<ServerDefaultSource>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub constant: Option<AttributeValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub player_attribute_reference: Option<AttributeReferenceInput>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateServerAttributeInput {
    pub universe_id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_type: Option<AttributeDataType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_value: Option<ServerDefaultValueInput>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateServerAttributeInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attribute_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_value: Option<ServerDefaultValueInput>,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct SignalWeightsInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invalid: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub occupancy: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub age: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferred_players: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub voice_chat: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_type: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub play_history: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_chat: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateScoringConfigurationInput {
    pub universe_id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matchmaking_signal_weights: Option<SignalWeightsInput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_signal_weights: Option<BTreeMap<String, f64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template: Option<ScoringTemplate>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateScoringConfigurationInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scoring_configuration_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matchmaking_signal_weights: Option<SignalWeightsInput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_signal_weights: Option<BTreeMap<String, f64>>,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerCategoricalSignalInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub player_attribute: Option<AttributeReferenceInput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub curve_type: Option<SignalCurveType>,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerCategoricalSignalInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_attribute: Option<AttributeReferenceInput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comparison_type: Option<CategoricalComparison>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub player_attribute: Option<AttributeReferenceInput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub constant_value: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerNumericalSignalInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_attribute: Option<AttributeReferenceInput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comparison_type: Option<NumericalComparison>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_relevant_difference: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub constant_value: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub player_attribute: Option<AttributeReferenceInput>,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AttributeAggregationInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aggregation_function: Option<AggregationFunction>,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerNumericalSignalInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub player_attribute: Option<AttributeReferenceInput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aggregation: Option<AttributeAggregationInput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comparison_type: Option<NumericalComparison>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_relevant_difference: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub constant_value: Option<f64>,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomSignalInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_signal_type: Option<CustomSignalType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub player_categorical_signal_configuration: Option<PlayerCategoricalSignalInput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_categorical_signal_configuration: Option<ServerCategoricalSignalInput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_numerical_signal_configuration: Option<ServerNumericalSignalInput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub player_numerical_signal_configuration: Option<PlayerNumericalSignalInput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCustomSignalInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scoring_configuration_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signal_configuration: Option<CustomSignalInput>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCustomSignalInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scoring_configuration_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signal_configuration: Option<CustomSignalInput>,
}

#[derive(Debug, Clone)]
pub struct MatchmakingConfigurationClient {
    transport: AsyncClient,
}

impl MatchmakingConfigurationClient {
    pub fn new(transport: AsyncClient) -> Self {
        Self { transport }
    }

    pub async fn create_server_attribute(
        &self,
        input: CreateServerAttributeInput,
    ) -> Result<Value, Error> {
        self.call(
            Operation::post(
                Server::Apis,
                "/matchmaking-api/v1/matchmaking/server-attribute",
            ),
            &[],
            &input,
        )
        .await
    }

    pub async fn update_server_attribute(
        &self,
        attribute_id: String,
        input: UpdateServerAttributeInput,
    ) -> Result<Value, Error> {
        self.call(
            Operation::patch(
                Server::Apis,
                "/matchmaking-api/v1/matchmaking/server-attribute/{attributeId}",
            ),
            &[("attributeId", attribute_id)],
            &input,
        )
        .await
    }

    pub async fn create_scoring_configuration(
        &self,
        input: CreateScoringConfigurationInput,
    ) -> Result<Value, Error> {
        self.call(
            Operation::post(
                Server::Apis,
                "/matchmaking-api/v1/matchmaking/scoring-configuration",
            ),
            &[],
            &input,
        )
        .await
    }

    pub async fn update_scoring_configuration(
        &self,
        id: String,
        input: UpdateScoringConfigurationInput,
    ) -> Result<Value, Error> {
        self.call(
            Operation::patch(
                Server::Apis,
                "/matchmaking-api/v1/matchmaking/scoring-configuration/{scoringConfigurationId}",
            ),
            &[("scoringConfigurationId", id)],
            &input,
        )
        .await
    }

    pub async fn create_custom_signal(
        &self,
        id: String,
        input: CreateCustomSignalInput,
    ) -> Result<Value, Error> {
        self.call(Operation::post(Server::Apis, "/matchmaking-api/v1/matchmaking/scoring-configuration/{scoringConfigurationId}/signals"), &[("scoringConfigurationId", id)], &input).await
    }

    pub async fn update_custom_signal(
        &self,
        id: String,
        signal_name: String,
        input: UpdateCustomSignalInput,
    ) -> Result<Value, Error> {
        self.call(Operation::patch(Server::Apis, "/matchmaking-api/v1/matchmaking/scoring-configuration/{scoringConfigurationId}/signals/{signalName}"), &[("scoringConfigurationId", id), ("signalName", signal_name)], &input).await
    }

    async fn call(
        &self,
        operation: Operation,
        params: &[(&'static str, String)],
        body: &impl Serialize,
    ) -> Result<Value, Error> {
        let mut request = Request::new(operation);
        for (name, value) in params {
            request = request.path_param(*name, value);
        }
        self.transport.execute(request.json(body)?).await
    }
}
