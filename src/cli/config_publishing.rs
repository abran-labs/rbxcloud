use clap::{Args, Subcommand};
use rbxcloud::rbx::{
    client::{AsyncClient, ClientPolicy},
    config_publishing::*,
    credentials::Credentials,
};

#[derive(Debug, Args)]
pub struct ConfigPublishing {
    #[command(subcommand)]
    command: Command,
}
#[derive(Debug, Subcommand)]
enum Command {
    UpdateDraft(DraftArgs),
    OverwriteDraft(DraftArgs),
    PublishDraft(PublishArgs),
}
#[derive(Debug, Args)]
struct DraftArgs {
    universe_id: String,
    repository: String,
    #[arg(long)]
    draft_hash: Option<String>,
    #[arg(long)]
    entries_json: Option<String>,
    #[arg(long)]
    conditional_rules_json: Option<String>,
    #[arg(long)]
    rules_order: Option<Vec<String>>,
}
#[derive(Debug, Args)]
struct PublishArgs {
    universe_id: String,
    repository: String,
    #[arg(long)]
    draft_hash: Option<String>,
    #[arg(long)]
    message: Option<String>,
    #[arg(long)]
    deployment_strategy: Option<DeploymentStrategy>,
}
impl DraftArgs {
    fn input(&self) -> anyhow::Result<DraftUpdateInput> {
        let rules = match &self.conditional_rules_json {
            Some(value) => Some(serde_json::from_str(value)?),
            None => None,
        };
        let conditional_rules =
            (rules.is_some() || self.rules_order.is_some()).then_some(ConditionalRulesInput {
                rules,
                rules_order: self.rules_order.clone(),
            });
        Ok(DraftUpdateInput {
            draft_hash: self.draft_hash.clone(),
            entries: self
                .entries_json
                .as_deref()
                .map(serde_json::from_str)
                .transpose()?,
            conditional_rules,
        })
    }
}
impl ConfigPublishing {
    pub async fn run(self) -> anyhow::Result<Option<String>> {
        let client = ConfigPublishingClient::new(
            AsyncClient::new(Credentials::from_process()?, ClientPolicy::default()).await?,
        );
        let value = match self.command {
            Command::UpdateDraft(args) => {
                client
                    .update(
                        args.universe_id.clone(),
                        args.repository.clone(),
                        args.input()?,
                    )
                    .await?
            }
            Command::OverwriteDraft(args) => {
                client
                    .overwrite(
                        args.universe_id.clone(),
                        args.repository.clone(),
                        args.input()?,
                    )
                    .await?
            }
            Command::PublishDraft(args) => {
                client
                    .publish(
                        args.universe_id,
                        args.repository,
                        DraftPublishInput {
                            draft_hash: args.draft_hash,
                            message: args.message,
                            deployment_strategy: args.deployment_strategy,
                        },
                    )
                    .await?
            }
        };
        Ok(Some(serde_json::to_string_pretty(&value)?))
    }
}
