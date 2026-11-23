use clap::{Args, Subcommand};
use rbxcloud::rbx::{
    analytics_alerts::*,
    analytics_dashboards::*,
    analytics_log_regexes::*,
    client::{AsyncClient, ClientPolicy},
    credentials::Credentials,
};

async fn transport() -> anyhow::Result<AsyncClient> {
    Ok(AsyncClient::new(Credentials::from_process()?, ClientPolicy::default()).await?)
}

#[derive(Debug, Args)]
pub struct AnalyticsDashboards {
    #[command(subcommand)]
    command: DashboardCommand,
}
#[derive(Debug, Subcommand)]
enum DashboardCommand {
    ValidateDocument(DocumentArgs),
    CreateDashboard(CreateDashboardArgs),
    UpdateMetadata(MetadataArgs),
    PublishDashboard(PublishArgs),
}
#[derive(Debug, Args)]
struct DocumentArgs {
    universe_id: u64,
    #[arg(long)]
    document_json: String,
}
#[derive(Debug, Args)]
struct CreateDashboardArgs {
    universe_id: u64,
    #[arg(long)]
    name: Option<String>,
    #[arg(long)]
    description: Option<String>,
    #[arg(long)]
    document_json: Option<String>,
    #[arg(long)]
    idempotency_key: Option<String>,
}
#[derive(Debug, Args, Default)]
struct MetadataFields {
    #[arg(long)]
    name: Option<String>,
    #[arg(long)]
    description: Option<String>,
    #[arg(long)]
    is_pinned: Option<bool>,
}
impl MetadataFields {
    fn build(self) -> Option<DashboardMetadataPatchInput> {
        (self.name.is_some() || self.description.is_some() || self.is_pinned.is_some()).then_some(
            DashboardMetadataPatchInput {
                name: self.name,
                description: self.description,
                is_pinned: self.is_pinned,
            },
        )
    }
}
#[derive(Debug, Args)]
struct MetadataArgs {
    universe_id: u64,
    dashboard_id: String,
    #[arg(long)]
    expected_head_etag: Option<String>,
    #[command(flatten)]
    patch: MetadataFields,
}
#[derive(Debug, Args)]
struct PublishArgs {
    universe_id: u64,
    dashboard_id: String,
    #[arg(long)]
    expected_head_etag: Option<String>,
    #[arg(long)]
    document_json: Option<String>,
    #[command(flatten)]
    metadata: MetadataFields,
}
fn document(value: Option<String>) -> anyhow::Result<Option<CustomDashboardDocumentInput>> {
    value
        .as_deref()
        .map(serde_json::from_str)
        .transpose()
        .map_err(Into::into)
}
impl AnalyticsDashboards {
    pub async fn run(self) -> anyhow::Result<Option<String>> {
        let client = AnalyticsDashboardClient::new(transport().await?);
        let value = match self.command {
            DashboardCommand::ValidateDocument(args) => {
                client
                    .validate_document(args.universe_id, serde_json::from_str(&args.document_json)?)
                    .await?
            }
            DashboardCommand::CreateDashboard(args) => {
                client
                    .create_dashboard(
                        args.universe_id,
                        CreateDashboardInput {
                            name: args.name,
                            description: args.description,
                            document: document(args.document_json)?,
                            idempotency_key: args.idempotency_key,
                        },
                    )
                    .await?
            }
            DashboardCommand::UpdateMetadata(args) => {
                client
                    .update_metadata(
                        args.universe_id,
                        args.dashboard_id,
                        UpdateDashboardMetadataInput {
                            expected_head_etag: args.expected_head_etag,
                            patch: args.patch.build(),
                        },
                    )
                    .await?
            }
            DashboardCommand::PublishDashboard(args) => {
                client
                    .publish_dashboard(
                        args.universe_id,
                        args.dashboard_id,
                        PublishDashboardInput {
                            expected_head_etag: args.expected_head_etag,
                            document: document(args.document_json)?,
                            metadata_patch: args.metadata.build(),
                        },
                    )
                    .await?
            }
        };
        Ok(Some(serde_json::to_string_pretty(&value)?))
    }
}

#[derive(Debug, Args)]
pub struct AnalyticsAlerts {
    #[command(subcommand)]
    command: AlertCommand,
}
#[derive(Debug, Subcommand)]
enum AlertCommand {
    CreateAlert(CreateAlertArgs),
    UpdateAlert(UpdateAlertArgs),
}
#[derive(Debug, Args)]
struct CreateAlertArgs {
    resource_type: String,
    resource_id: String,
    #[arg(long)]
    name: String,
    #[arg(long)]
    metric: String,
    #[arg(long)]
    description: Option<String>,
    #[arg(long)]
    severity: AlertSeverity,
    #[arg(long)]
    interval: AlertInterval,
    #[arg(long)]
    consecutive_occurrences: u32,
    #[arg(long)]
    breakdown_json: Option<String>,
    #[arg(long = "filter", value_parser = parse_filter)]
    filters: Vec<AlertFilterInput>,
    #[arg(long)]
    condition_operator: ConditionOperator,
    #[arg(long, allow_negative_numbers = true)]
    condition_threshold: f64,
    #[arg(long)]
    evaluation_mode: EvaluationMode,
    #[arg(long)]
    period_offset_multiplier: Option<u32>,
    #[arg(long)]
    webhook_configuration_id: Vec<String>,
}
#[derive(Debug, Args)]
struct UpdateAlertArgs {
    resource_type: String,
    resource_id: String,
    alert_id: String,
    #[arg(long)]
    name: Option<String>,
    #[arg(long)]
    metric: Option<String>,
    #[arg(long)]
    description: Option<String>,
    #[arg(long)]
    severity: Option<AlertSeverity>,
    #[arg(long)]
    interval: Option<AlertInterval>,
    #[arg(long)]
    consecutive_occurrences: Option<u32>,
    #[arg(long)]
    breakdown_json: Option<String>,
    #[arg(long = "filter", value_parser = parse_filter)]
    filters: Vec<AlertFilterInput>,
    #[arg(long)]
    condition_operator: Option<ConditionOperator>,
    #[arg(long, allow_negative_numbers = true)]
    condition_threshold: Option<f64>,
    #[arg(long)]
    evaluation_mode: Option<EvaluationMode>,
    #[arg(long)]
    period_offset_multiplier: Option<u32>,
    #[arg(long)]
    webhook_configuration_id: Vec<String>,
    #[arg(long, conflicts_with = "webhook_configuration_id")]
    clear_webhook_receivers: bool,
    #[arg(long)]
    config_state: Option<AlertConfigState>,
}
fn parse_filter(value: &str) -> Result<AlertFilterInput, String> {
    let (dimension, values) = value
        .split_once('=')
        .ok_or("expected DIMENSION=VALUE[,VALUE]")?;
    if dimension.is_empty() {
        return Err("dimension cannot be empty".into());
    }
    Ok(AlertFilterInput {
        dimension: dimension.into(),
        values: values.split(',').map(str::to_string).collect(),
    })
}
fn breakdown(value: Option<String>) -> anyhow::Result<Option<Vec<AlertBreakdownInput>>> {
    value
        .as_deref()
        .map(serde_json::from_str)
        .transpose()
        .map_err(Into::into)
}
fn receivers(ids: Vec<String>) -> Option<WebhookReceiverConfigInput> {
    (!ids.is_empty()).then_some(WebhookReceiverConfigInput {
        receivers: Some(
            ids.into_iter()
                .map(|webhook_configuration_id| WebhookReceiverInput {
                    webhook_configuration_id,
                })
                .collect(),
        ),
    })
}
impl AnalyticsAlerts {
    pub async fn run(self) -> anyhow::Result<Option<String>> {
        let client = AnalyticsAlertClient::new(transport().await?);
        let value = match self.command {
            AlertCommand::CreateAlert(args) => {
                client
                    .create_alert(
                        &args.resource_type,
                        &args.resource_id,
                        CreateAlertInput {
                            name: args.name,
                            metric: args.metric,
                            description: args.description,
                            severity: args.severity,
                            interval: args.interval,
                            consecutive_occurrences: args.consecutive_occurrences,
                            breakdown: breakdown(args.breakdown_json)?,
                            filter: (!args.filters.is_empty()).then_some(args.filters),
                            condition: AlertConditionInput {
                                operator: Some(args.condition_operator),
                                threshold: Some(args.condition_threshold),
                                evaluation_mode: Some(args.evaluation_mode),
                                period_offset_multiplier: args.period_offset_multiplier,
                            },
                            webhook_receiver_config: receivers(args.webhook_configuration_id),
                        },
                    )
                    .await?
            }
            AlertCommand::UpdateAlert(args) => {
                let condition = (args.condition_operator.is_some()
                    || args.condition_threshold.is_some()
                    || args.evaluation_mode.is_some()
                    || args.period_offset_multiplier.is_some())
                .then_some(AlertConditionInput {
                    operator: args.condition_operator,
                    threshold: args.condition_threshold,
                    evaluation_mode: args.evaluation_mode,
                    period_offset_multiplier: args.period_offset_multiplier,
                });
                let webhook_receiver_config = if args.clear_webhook_receivers {
                    Some(None)
                } else {
                    receivers(args.webhook_configuration_id).map(Some)
                };
                client
                    .update_alert(
                        &args.resource_type,
                        &args.resource_id,
                        &args.alert_id,
                        UpdateAlertInput {
                            name: args.name,
                            metric: args.metric,
                            description: args.description,
                            severity: args.severity,
                            interval: args.interval,
                            consecutive_occurrences: args.consecutive_occurrences,
                            breakdown: breakdown(args.breakdown_json)?,
                            filter: (!args.filters.is_empty()).then_some(args.filters),
                            condition,
                            webhook_receiver_config,
                            config_state: args.config_state,
                        },
                    )
                    .await?
            }
        };
        Ok(Some(serde_json::to_string_pretty(&value)?))
    }
}

#[derive(Debug, Args)]
pub struct AnalyticsLogRegexes {
    #[command(subcommand)]
    command: LogRegexCommand,
}
#[derive(Debug, Subcommand)]
enum LogRegexCommand {
    CreateRegex(CreateRegexArgs),
    UpdateRegex(UpdateRegexArgs),
}
#[derive(Debug, Args)]
struct CreateRegexArgs {
    #[arg(long)]
    universe_id: u64,
    #[arg(long)]
    pattern: Option<String>,
    #[arg(long)]
    output: Option<String>,
    #[arg(long)]
    regex_operation: Option<RegexOperation>,
}
#[derive(Debug, Args)]
struct UpdateRegexArgs {
    id: String,
    #[arg(long)]
    pattern: Option<String>,
    #[arg(long)]
    output: Option<String>,
    #[arg(long)]
    regex_operation: Option<RegexOperation>,
}
impl AnalyticsLogRegexes {
    pub async fn run(self) -> anyhow::Result<Option<String>> {
        let client = AnalyticsLogRegexClient::new(transport().await?);
        let value = match self.command {
            LogRegexCommand::CreateRegex(args) => {
                client
                    .create_regex(CreateLogRegexInput {
                        universe_id: args.universe_id,
                        pattern: args.pattern,
                        output: args.output,
                        regex_operation: args.regex_operation,
                    })
                    .await?
            }
            LogRegexCommand::UpdateRegex(args) => {
                client
                    .update_regex(
                        &args.id,
                        UpdateLogRegexInput {
                            pattern: args.pattern,
                            output: args.output,
                            regex_operation: args.regex_operation,
                        },
                    )
                    .await?
            }
        };
        Ok(Some(serde_json::to_string_pretty(&value)?))
    }
}
