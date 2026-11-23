use clap::{Args, Subcommand};
use rbxcloud::rbx::{
    campaign_management::*,
    campaign_planning::{AudienceType, CampaignObjective},
    client::{AsyncClient, ClientPolicy},
    credentials::Credentials,
};

#[derive(Debug, Args)]
pub struct CampaignManagement {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    EstimateAudience(AudienceArgs),
    CreateCampaign(CampaignArgs),
    UpdateCampaign(UpdateArgs),
    ValidateCampaign(CampaignArgs),
    EstimatePerformance(PerformanceArgs),
}

#[derive(Debug, Args, Default)]
struct TargetingArgs {
    #[arg(long, value_delimiter = ',')]
    age_groups: Vec<AgeGroup>,
    #[arg(long)]
    audience_type: Option<AudienceType>,
    #[arg(long, value_delimiter = ',')]
    countries: Vec<String>,
    #[arg(long, value_delimiter = ',')]
    devices: Vec<Device>,
    #[arg(long, value_delimiter = ',')]
    regions: Vec<Region>,
}

impl TargetingArgs {
    fn build(self) -> Option<TargetingInput> {
        let present = !self.age_groups.is_empty()
            || self.audience_type.is_some()
            || !self.countries.is_empty()
            || !self.devices.is_empty()
            || !self.regions.is_empty();
        present.then_some(TargetingInput {
            age_groups: (!self.age_groups.is_empty()).then_some(self.age_groups),
            audience_type: self.audience_type,
            countries: (!self.countries.is_empty()).then_some(self.countries),
            devices: (!self.devices.is_empty()).then_some(self.devices),
            regions: (!self.regions.is_empty()).then_some(self.regions),
        })
    }
}

#[derive(Debug, Args)]
struct AudienceArgs {
    #[arg(long)]
    objective: CampaignObjective,
    #[arg(long)]
    target_universe_id: String,
    #[command(flatten)]
    targeting: TargetingArgs,
}

#[derive(Debug, Args, Default)]
struct BudgetArgs {
    #[arg(long, requires = "budget_type")]
    budget_amount_micros: Option<String>,
    #[arg(long, requires = "budget_amount_micros")]
    budget_type: Option<BudgetType>,
}

impl BudgetArgs {
    fn build(self) -> Option<BudgetInput> {
        self.budget_amount_micros.map(|amount_micros| BudgetInput {
            amount_micros,
            budget_type: self.budget_type.expect("clap requires budget type"),
        })
    }
}

#[derive(Debug, Args, Default)]
struct ScheduleArgs {
    #[arg(long, requires = "schedule_start_time")]
    schedule_duration_in_days: Option<u32>,
    #[arg(long, requires = "schedule_duration_in_days")]
    schedule_start_time: Option<String>,
}

impl ScheduleArgs {
    fn build(self) -> Option<ScheduleInput> {
        self.schedule_duration_in_days
            .map(|duration_in_days| ScheduleInput {
                duration_in_days,
                start_time: self.schedule_start_time.expect("clap requires start time"),
            })
    }
}

#[derive(Debug, Args, Default)]
struct CampaignFields {
    #[arg(long)]
    auto_reload_ad_credit_enabled: Option<bool>,
    #[arg(long)]
    billing_account_id: Option<String>,
    #[command(flatten)]
    budget: BudgetArgs,
    #[arg(long, value_delimiter = ',')]
    creative_asset_ids: Vec<String>,
    #[arg(long)]
    idempotency_key: Option<String>,
    #[arg(long)]
    name: Option<String>,
    #[arg(long)]
    objective: Option<CampaignObjective>,
    #[arg(long)]
    payment_type: Option<PaymentType>,
    #[command(flatten)]
    schedule: ScheduleArgs,
    #[arg(long)]
    target_universe_id: Option<String>,
    #[command(flatten)]
    targeting: TargetingArgs,
    #[arg(long)]
    video_source: Option<VideoSource>,
}

impl CampaignFields {
    fn build(self) -> CampaignInput {
        CampaignInput {
            auto_reload_ad_credit_enabled: self.auto_reload_ad_credit_enabled,
            billing_account_id: self.billing_account_id,
            budget: self.budget.build(),
            creative_asset_ids: (!self.creative_asset_ids.is_empty())
                .then_some(self.creative_asset_ids),
            idempotency_key: self.idempotency_key,
            name: self.name,
            objective: self.objective,
            payment_type: self.payment_type,
            schedule: self.schedule.build(),
            target_universe_id: self.target_universe_id,
            targeting: self.targeting.build(),
            video: self.video_source.map(|source| VideoInput { source }),
        }
    }
}

#[derive(Debug, Args)]
struct CampaignArgs {
    #[command(flatten)]
    fields: CampaignFields,
}

#[derive(Debug, Args)]
struct UpdateArgs {
    id: String,
    #[arg(long)]
    auto_reload_ad_credit_enabled: Option<bool>,
    #[arg(long)]
    budget_amount_micros: Option<String>,
    #[arg(long, value_delimiter = ',')]
    creative_asset_ids: Vec<String>,
    #[arg(long)]
    name: Option<String>,
    #[arg(long)]
    schedule_duration_in_days: Option<u32>,
    #[arg(long)]
    schedule_start_time: Option<String>,
    #[arg(long)]
    status: Option<CampaignStatus>,
    #[arg(long)]
    video_source: Option<VideoSource>,
}

#[derive(Debug, Args)]
struct PerformanceArgs {
    #[arg(long)]
    target_universe_id: String,
    #[arg(long, conflicts_with = "target_outcome_value")]
    budget_amount_micro_usd: Option<String>,
    #[arg(long, conflicts_with = "budget_amount_micro_usd")]
    target_outcome_value: Option<u64>,
    #[arg(long)]
    budget_type: BudgetType,
    #[arg(long)]
    duration_in_days: Option<u32>,
}

impl CampaignManagement {
    pub async fn run(self) -> anyhow::Result<Option<String>> {
        let client = CampaignClient::new(
            AsyncClient::new(Credentials::from_process()?, ClientPolicy::default()).await?,
        );
        let value = match self.command {
            Command::EstimateAudience(args) => {
                client
                    .estimate_audience(AudienceEstimateInput {
                        objective: args.objective,
                        target_universe_id: args.target_universe_id,
                        targeting: args.targeting.build(),
                    })
                    .await?
            }
            Command::CreateCampaign(args) => client.create(args.fields.build()).await?,
            Command::ValidateCampaign(args) => client.validate(args.fields.build()).await?,
            Command::UpdateCampaign(args) => {
                let schedule = (args.schedule_duration_in_days.is_some()
                    || args.schedule_start_time.is_some())
                .then_some(UpdateScheduleInput {
                    duration_in_days: args.schedule_duration_in_days,
                    start_time: args.schedule_start_time,
                });
                client
                    .update(
                        args.id,
                        CampaignUpdateInput {
                            auto_reload_ad_credit_enabled: args.auto_reload_ad_credit_enabled,
                            budget: args
                                .budget_amount_micros
                                .map(|amount_micros| UpdateBudgetInput { amount_micros }),
                            creative_asset_ids: (!args.creative_asset_ids.is_empty())
                                .then_some(args.creative_asset_ids),
                            name: args.name,
                            schedule,
                            status: args.status,
                            video: args.video_source.map(|source| VideoInput { source }),
                        },
                    )
                    .await?
            }
            Command::EstimatePerformance(args) => {
                client
                    .estimate_performance(PerformanceEstimateInput {
                        objective: PerformanceObjective::Plays,
                        target_universe_id: args.target_universe_id,
                        forecast_input: PerformanceForecastInput {
                            budget: args.budget_amount_micro_usd.map(|amount_micro_usd| {
                                PerformanceBudgetInput {
                                    amount_micro_usd: Some(amount_micro_usd),
                                }
                            }),
                            target_outcome: args.target_outcome_value.map(|value| {
                                PerformanceTargetOutcome {
                                    outcome_type: PerformanceObjective::Plays,
                                    value: Some(value),
                                }
                            }),
                        },
                        forecast_scope: PerformanceScope {
                            budget_type: args.budget_type,
                            duration_in_days: args.duration_in_days,
                        },
                    })
                    .await?
            }
        };
        Ok(Some(serde_json::to_string_pretty(&value)?))
    }
}
