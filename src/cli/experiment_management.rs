use clap::{Args, Subcommand};
use rbxcloud::rbx::{
    client::{AsyncClient, ClientPolicy},
    credentials::Credentials,
    experiment_management::*,
};

#[derive(Debug, Args)]
pub struct Experiments {
    #[command(subcommand)]
    command: Command,
}
#[derive(Debug, Subcommand)]
enum Command {
    CreateExperiment(ExperimentArgs),
    UpdateExperiment(UpdateArgs),
    CalculateExperimentMde(MdeArgs),
}
#[derive(Debug, Args)]
struct ExperimentArgs {
    universe_id: u64,
    #[arg(long)]
    name: Option<String>,
    #[arg(long)]
    description: Option<String>,
    #[arg(long)]
    configuration_json: Option<String>,
    #[arg(long)]
    exposure_percent: Option<u8>,
    #[arg(long)]
    targeting_rule_json: Option<String>,
    #[arg(long)]
    duration_seconds: Option<u64>,
    #[arg(long)]
    universe_goal_metric: Option<GoalMetric>,
}
#[derive(Debug, Args)]
struct UpdateArgs {
    universe_id: u64,
    experiment_id: String,
    #[command(flatten)]
    experiment: ExperimentFields,
}
#[derive(Debug, Args)]
struct ExperimentFields {
    #[arg(long)]
    name: Option<String>,
    #[arg(long)]
    description: Option<String>,
    #[arg(long)]
    configuration_json: Option<String>,
    #[arg(long)]
    exposure_percent: Option<u8>,
    #[arg(long)]
    targeting_rule_json: Option<String>,
    #[arg(long)]
    duration_seconds: Option<u64>,
    #[arg(long)]
    universe_goal_metric: Option<GoalMetric>,
}
#[derive(Debug, Args)]
struct MdeArgs {
    universe_id: u64,
    #[arg(long)]
    exposure_percent: Option<u8>,
    #[arg(long)]
    duration_seconds: Option<u64>,
    #[arg(long, allow_negative_numbers = true)]
    baseline_weight: Option<f64>,
    #[arg(long, allow_negative_numbers = true)]
    variant_weights: Option<Vec<f64>>,
    #[arg(long)]
    universe_goal_metric: Option<GoalMetric>,
    #[arg(long)]
    targeting_rule_json: Option<String>,
}
fn experiment_input(fields: ExperimentFields) -> anyhow::Result<ExperimentInput> {
    Ok(ExperimentInput {
        name: fields.name,
        description: fields.description,
        experiment_configuration: fields
            .configuration_json
            .as_deref()
            .map(serde_json::from_str)
            .transpose()?,
        exposure_percent: fields.exposure_percent,
        targeting_criteria: fields
            .targeting_rule_json
            .as_deref()
            .map(|value| {
                serde_json::from_str::<serde_json::Value>(value)
                    .map(|rule| serde_json::json!({"rule":rule}))
            })
            .transpose()?,
        duration_seconds: fields.duration_seconds,
        universe_goal_metric: fields.universe_goal_metric,
    })
}
impl Experiments {
    pub async fn run(self) -> anyhow::Result<Option<String>> {
        let client = ExperimentClient::new(
            AsyncClient::new(Credentials::from_process()?, ClientPolicy::default()).await?,
        );
        let value = match self.command {
            Command::CreateExperiment(args) => {
                client
                    .create(
                        args.universe_id,
                        experiment_input(ExperimentFields {
                            name: args.name,
                            description: args.description,
                            configuration_json: args.configuration_json,
                            exposure_percent: args.exposure_percent,
                            targeting_rule_json: args.targeting_rule_json,
                            duration_seconds: args.duration_seconds,
                            universe_goal_metric: args.universe_goal_metric,
                        })?,
                    )
                    .await?
            }
            Command::UpdateExperiment(args) => {
                client
                    .update(
                        args.universe_id,
                        args.experiment_id,
                        experiment_input(args.experiment)?,
                    )
                    .await?
            }
            Command::CalculateExperimentMde(args) => {
                client
                    .calculate_mde(
                        args.universe_id,
                        MdeInput {
                            exposure_percent: args.exposure_percent,
                            duration_seconds: args.duration_seconds,
                            baseline_weight: args.baseline_weight,
                            variant_weights: args.variant_weights,
                            universe_goal_metric: args.universe_goal_metric,
                            targeting_criteria: args
                                .targeting_rule_json
                                .as_deref()
                                .map(|value| {
                                    serde_json::from_str::<serde_json::Value>(value)
                                        .map(|rule| serde_json::json!({"rule":rule}))
                                })
                                .transpose()?,
                        },
                    )
                    .await?
            }
        };
        Ok(Some(serde_json::to_string_pretty(&value)?))
    }
}
