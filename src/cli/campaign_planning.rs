use clap::Args;
use rbxcloud::rbx::{
    campaign_planning::*,
    client::{AsyncClient, ClientPolicy},
    credentials::Credentials,
};

#[derive(Debug, Args)]
pub struct CampaignRecommendation {
    #[command(subcommand)]
    command: Recommend,
}
#[derive(Debug, clap::Subcommand)]
enum Recommend {
    Recommend(RecommendationArgs),
}
#[derive(Debug, Args)]
struct RecommendationArgs {
    #[arg(long)]
    objective: CampaignObjective,
    #[arg(long)]
    target_universe_id: String,
    #[arg(long)]
    audience_type: Option<AudienceType>,
}
impl CampaignRecommendation {
    pub async fn run(self) -> anyhow::Result<Option<String>> {
        let Recommend::Recommend(this) = self.command;
        let client = CampaignPlanningClient::new(
            AsyncClient::new(Credentials::from_process()?, ClientPolicy::default()).await?,
        );
        let targeting = this.audience_type.map(|audience_type| PlanningTargeting {
            audience_type: Some(audience_type),
        });
        let result = client
            .recommend(PlanningContext {
                objective: this.objective,
                target_universe_id: this.target_universe_id,
                targeting,
            })
            .await?;
        Ok(Some(serde_json::to_string_pretty(&result)?))
    }
}
