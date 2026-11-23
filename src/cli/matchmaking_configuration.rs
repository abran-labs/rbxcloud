use std::collections::BTreeMap;

use clap::{Args, Subcommand};
use rbxcloud::rbx::{
    client::{AsyncClient, ClientPolicy},
    credentials::Credentials,
    matchmaking_attributes::{AttributeDataType, AttributeValue},
    matchmaking_configuration::*,
};

async fn client() -> anyhow::Result<MatchmakingConfigurationClient> {
    Ok(MatchmakingConfigurationClient::new(
        AsyncClient::new(Credentials::from_process()?, ClientPolicy::default()).await?,
    ))
}

#[derive(Debug, Args)]
pub struct ServerAttributes {
    #[command(subcommand)]
    command: ServerAttributeCommand,
}

#[derive(Debug, Subcommand)]
enum ServerAttributeCommand {
    CreateServerAttribute(CreateServerArgs),
    UpdateServerAttribute(UpdateServerArgs),
}

#[derive(Debug, Args, Default)]
struct ServerDefaultArgs {
    #[arg(long)]
    default_source: Option<ServerDefaultSource>,
    #[arg(long, conflicts_with_all = ["default_numeric_value", "default_string_value"])]
    default_bool_value: Option<bool>,
    #[arg(long, conflicts_with_all = ["default_bool_value", "default_string_value"], allow_negative_numbers = true)]
    default_numeric_value: Option<f64>,
    #[arg(long, conflicts_with_all = ["default_bool_value", "default_numeric_value"])]
    default_string_value: Option<String>,
    #[arg(long)]
    default_value_type: Option<AttributeDataType>,
    #[arg(long)]
    player_attribute_reference_id: Option<String>,
}

impl ServerDefaultArgs {
    fn build(self) -> Option<ServerDefaultValueInput> {
        let constant = (self.default_bool_value.is_some()
            || self.default_numeric_value.is_some()
            || self.default_string_value.is_some()
            || self.default_value_type.is_some())
        .then_some(AttributeValue {
            bool_value: self.default_bool_value,
            numeric_value: self.default_numeric_value,
            string_value: self.default_string_value,
            value_type: self.default_value_type,
        });
        let player_attribute_reference = self
            .player_attribute_reference_id
            .map(|id| AttributeReferenceInput { id: Some(id) });
        (self.default_source.is_some()
            || constant.is_some()
            || player_attribute_reference.is_some())
        .then_some(ServerDefaultValueInput {
            source_case: self.default_source,
            constant,
            player_attribute_reference,
        })
    }
}

#[derive(Debug, Args)]
struct CreateServerArgs {
    #[arg(long)]
    universe_id: u64,
    #[arg(long)]
    name: Option<String>,
    #[arg(long)]
    data_type: Option<AttributeDataType>,
    #[command(flatten)]
    default: ServerDefaultArgs,
}

#[derive(Debug, Args)]
struct UpdateServerArgs {
    attribute_id: String,
    #[command(flatten)]
    default: ServerDefaultArgs,
}

impl ServerAttributes {
    pub async fn run(self) -> anyhow::Result<Option<String>> {
        let client = client().await?;
        let value = match self.command {
            ServerAttributeCommand::CreateServerAttribute(args) => {
                client
                    .create_server_attribute(CreateServerAttributeInput {
                        universe_id: args.universe_id,
                        name: args.name,
                        data_type: args.data_type,
                        default_value: args.default.build(),
                    })
                    .await?
            }
            ServerAttributeCommand::UpdateServerAttribute(args) => {
                client
                    .update_server_attribute(
                        args.attribute_id.clone(),
                        UpdateServerAttributeInput {
                            attribute_id: Some(args.attribute_id),
                            default_value: args.default.build(),
                        },
                    )
                    .await?
            }
        };
        Ok(Some(serde_json::to_string_pretty(&value)?))
    }
}

#[derive(Debug, Args)]
pub struct ScoringConfigurations {
    #[command(subcommand)]
    command: ScoringCommand,
}

#[derive(Debug, Subcommand)]
enum ScoringCommand {
    CreateScoringConfiguration(CreateScoringArgs),
    UpdateScoringConfiguration(UpdateScoringArgs),
}

#[derive(Debug, Args, Default)]
struct SignalWeightArgs {
    #[arg(long, allow_negative_numbers = true)]
    invalid_weight: Option<f64>,
    #[arg(long, allow_negative_numbers = true)]
    occupancy_weight: Option<f64>,
    #[arg(long, allow_negative_numbers = true)]
    age_weight: Option<f64>,
    #[arg(long, allow_negative_numbers = true)]
    language_weight: Option<f64>,
    #[arg(long, allow_negative_numbers = true)]
    latency_weight: Option<f64>,
    #[arg(long, allow_negative_numbers = true)]
    preferred_players_weight: Option<f64>,
    #[arg(long, allow_negative_numbers = true)]
    voice_chat_weight: Option<f64>,
    #[arg(long, allow_negative_numbers = true)]
    device_type_weight: Option<f64>,
    #[arg(long, allow_negative_numbers = true)]
    play_history_weight: Option<f64>,
    #[arg(long, allow_negative_numbers = true)]
    text_chat_weight: Option<f64>,
    #[arg(long = "custom-signal-weight", value_parser = parse_custom_weight)]
    custom_signal_weights: Vec<(String, f64)>,
}

impl SignalWeightArgs {
    fn standard(&self) -> Option<SignalWeightsInput> {
        let present = self.invalid_weight.is_some()
            || self.occupancy_weight.is_some()
            || self.age_weight.is_some()
            || self.language_weight.is_some()
            || self.latency_weight.is_some()
            || self.preferred_players_weight.is_some()
            || self.voice_chat_weight.is_some()
            || self.device_type_weight.is_some()
            || self.play_history_weight.is_some()
            || self.text_chat_weight.is_some();
        present.then_some(SignalWeightsInput {
            invalid: self.invalid_weight,
            occupancy: self.occupancy_weight,
            age: self.age_weight,
            language: self.language_weight,
            latency: self.latency_weight,
            preferred_players: self.preferred_players_weight,
            voice_chat: self.voice_chat_weight,
            device_type: self.device_type_weight,
            play_history: self.play_history_weight,
            text_chat: self.text_chat_weight,
        })
    }

    fn custom(&self) -> Option<BTreeMap<String, f64>> {
        (!self.custom_signal_weights.is_empty())
            .then(|| self.custom_signal_weights.iter().cloned().collect())
    }
}

fn parse_custom_weight(value: &str) -> Result<(String, f64), String> {
    let (name, weight) = value.split_once('=').ok_or("expected NAME=WEIGHT")?;
    if name.is_empty() {
        return Err("signal name cannot be empty".into());
    }
    let weight = weight
        .parse::<f64>()
        .map_err(|_| "WEIGHT must be a number")?;
    Ok((name.into(), weight))
}

#[derive(Debug, Args)]
struct CreateScoringArgs {
    #[arg(long)]
    universe_id: u64,
    #[arg(long)]
    name: Option<String>,
    #[command(flatten)]
    weights: SignalWeightArgs,
    #[arg(long)]
    template: Option<ScoringTemplate>,
}

#[derive(Debug, Args)]
struct UpdateScoringArgs {
    scoring_configuration_id: String,
    #[arg(long)]
    name: Option<String>,
    #[command(flatten)]
    weights: SignalWeightArgs,
}

impl ScoringConfigurations {
    pub async fn run(self) -> anyhow::Result<Option<String>> {
        let client = client().await?;
        let value = match self.command {
            ScoringCommand::CreateScoringConfiguration(args) => {
                client
                    .create_scoring_configuration(CreateScoringConfigurationInput {
                        universe_id: args.universe_id,
                        name: args.name,
                        matchmaking_signal_weights: args.weights.standard(),
                        custom_signal_weights: args.weights.custom(),
                        template: args.template,
                    })
                    .await?
            }
            ScoringCommand::UpdateScoringConfiguration(args) => {
                client
                    .update_scoring_configuration(
                        args.scoring_configuration_id.clone(),
                        UpdateScoringConfigurationInput {
                            scoring_configuration_id: Some(args.scoring_configuration_id),
                            name: args.name,
                            matchmaking_signal_weights: args.weights.standard(),
                            custom_signal_weights: args.weights.custom(),
                        },
                    )
                    .await?
            }
        };
        Ok(Some(serde_json::to_string_pretty(&value)?))
    }
}

#[derive(Debug, Args)]
pub struct CustomSignals {
    #[command(subcommand)]
    command: CustomSignalCommand,
}

#[derive(Debug, Subcommand)]
enum CustomSignalCommand {
    CreateCustomSignal(CreateSignalArgs),
    UpdateCustomSignal(UpdateSignalArgs),
}

#[derive(Debug, Args, Default)]
struct CustomSignalFields {
    #[arg(long)]
    name: Option<String>,
    #[arg(long, allow_negative_numbers = true)]
    weight: Option<f64>,
    #[arg(long)]
    custom_signal_type: Option<CustomSignalType>,
    #[arg(long)]
    description: Option<String>,
    #[arg(long)]
    player_categorical_attribute_id: Option<String>,
    #[arg(long)]
    player_categorical_curve_type: Option<SignalCurveType>,
    #[arg(long)]
    server_categorical_server_attribute_id: Option<String>,
    #[arg(long)]
    server_categorical_player_attribute_id: Option<String>,
    #[arg(long)]
    categorical_comparison_type: Option<CategoricalComparison>,
    #[arg(long)]
    categorical_constant_value: Option<String>,
    #[arg(long)]
    server_numerical_server_attribute_id: Option<String>,
    #[arg(long)]
    server_numerical_player_attribute_id: Option<String>,
    #[arg(long)]
    server_numerical_comparison_type: Option<NumericalComparison>,
    #[arg(long, allow_negative_numbers = true)]
    server_numerical_max_relevant_difference: Option<f64>,
    #[arg(long, allow_negative_numbers = true)]
    server_numerical_constant_value: Option<f64>,
    #[arg(long)]
    player_numerical_attribute_id: Option<String>,
    #[arg(long)]
    aggregation_function: Option<AggregationFunction>,
    #[arg(long)]
    player_numerical_comparison_type: Option<NumericalComparison>,
    #[arg(long, allow_negative_numbers = true)]
    player_numerical_max_relevant_difference: Option<f64>,
    #[arg(long, allow_negative_numbers = true)]
    player_numerical_constant_value: Option<f64>,
}

fn reference(id: Option<String>) -> Option<AttributeReferenceInput> {
    id.map(|id| AttributeReferenceInput { id: Some(id) })
}

impl CustomSignalFields {
    fn build(self) -> CustomSignalInput {
        let player_categorical = (self.player_categorical_attribute_id.is_some()
            || self.player_categorical_curve_type.is_some())
        .then_some(PlayerCategoricalSignalInput {
            player_attribute: reference(self.player_categorical_attribute_id),
            curve_type: self.player_categorical_curve_type,
        });
        let server_categorical = (self.server_categorical_server_attribute_id.is_some()
            || self.server_categorical_player_attribute_id.is_some()
            || self.categorical_comparison_type.is_some()
            || self.categorical_constant_value.is_some())
        .then_some(ServerCategoricalSignalInput {
            server_attribute: reference(self.server_categorical_server_attribute_id),
            comparison_type: self.categorical_comparison_type,
            player_attribute: reference(self.server_categorical_player_attribute_id),
            constant_value: self.categorical_constant_value,
        });
        let server_numerical = (self.server_numerical_server_attribute_id.is_some()
            || self.server_numerical_player_attribute_id.is_some()
            || self.server_numerical_comparison_type.is_some()
            || self.server_numerical_max_relevant_difference.is_some()
            || self.server_numerical_constant_value.is_some())
        .then_some(ServerNumericalSignalInput {
            server_attribute: reference(self.server_numerical_server_attribute_id),
            comparison_type: self.server_numerical_comparison_type,
            max_relevant_difference: self.server_numerical_max_relevant_difference,
            constant_value: self.server_numerical_constant_value,
            player_attribute: reference(self.server_numerical_player_attribute_id),
        });
        let player_numerical = (self.player_numerical_attribute_id.is_some()
            || self.aggregation_function.is_some()
            || self.player_numerical_comparison_type.is_some()
            || self.player_numerical_max_relevant_difference.is_some()
            || self.player_numerical_constant_value.is_some())
        .then_some(PlayerNumericalSignalInput {
            player_attribute: reference(self.player_numerical_attribute_id),
            aggregation: self.aggregation_function.map(|aggregation_function| {
                AttributeAggregationInput {
                    aggregation_function: Some(aggregation_function),
                }
            }),
            comparison_type: self.player_numerical_comparison_type,
            max_relevant_difference: self.player_numerical_max_relevant_difference,
            constant_value: self.player_numerical_constant_value,
        });
        CustomSignalInput {
            name: self.name,
            weight: self.weight,
            custom_signal_type: self.custom_signal_type,
            player_categorical_signal_configuration: player_categorical,
            server_categorical_signal_configuration: server_categorical,
            server_numerical_signal_configuration: server_numerical,
            player_numerical_signal_configuration: player_numerical,
            description: self.description,
        }
    }
}

#[derive(Debug, Args)]
struct CreateSignalArgs {
    scoring_configuration_id: String,
    #[command(flatten)]
    fields: CustomSignalFields,
}

#[derive(Debug, Args)]
struct UpdateSignalArgs {
    scoring_configuration_id: String,
    signal_name: String,
    #[command(flatten)]
    fields: CustomSignalFields,
}

impl CustomSignals {
    pub async fn run(self) -> anyhow::Result<Option<String>> {
        let client = client().await?;
        let value = match self.command {
            CustomSignalCommand::CreateCustomSignal(args) => {
                client
                    .create_custom_signal(
                        args.scoring_configuration_id.clone(),
                        CreateCustomSignalInput {
                            scoring_configuration_id: Some(args.scoring_configuration_id),
                            signal_configuration: Some(args.fields.build()),
                        },
                    )
                    .await?
            }
            CustomSignalCommand::UpdateCustomSignal(args) => {
                client
                    .update_custom_signal(
                        args.scoring_configuration_id.clone(),
                        args.signal_name,
                        UpdateCustomSignalInput {
                            scoring_configuration_id: Some(args.scoring_configuration_id),
                            signal_configuration: Some(args.fields.build()),
                        },
                    )
                    .await?
            }
        };
        Ok(Some(serde_json::to_string_pretty(&value)?))
    }
}
