use clap::{Args, Subcommand};
use rbxcloud::rbx::{
    asset_permission_grants::*,
    client::{AsyncClient, ClientPolicy},
    credentials::Credentials,
    economy_uploads::*,
};
use std::path::PathBuf;

async fn transport() -> anyhow::Result<AsyncClient> {
    Ok(AsyncClient::new(Credentials::from_process()?, ClientPolicy::default()).await?)
}

#[derive(Debug, Args)]
pub struct EconomyUploads {
    #[command(subcommand)]
    command: Command,
}
#[derive(Debug, Subcommand)]
enum Command {
    CreateDeveloperProduct(CreateDeveloperArgs),
    UpdateDeveloperProduct(UpdateDeveloperArgs),
    CreateGamePass(CreatePassArgs),
    UpdateGamePass(UpdatePassArgs),
    CreateBadge(CreateBadgeArgs),
    UpdateBadgeIcon(BadgeIconArgs),
}
#[derive(Debug, Args)]
struct CreateDeveloperArgs {
    universe_id: u64,
    #[arg(long)]
    name: String,
    #[arg(long)]
    description: Option<String>,
    #[arg(long)]
    is_for_sale: Option<bool>,
    #[arg(long)]
    price: Option<i64>,
    #[arg(long)]
    image_file: Option<PathBuf>,
    #[arg(long, default_value = "image/png")]
    content_type: String,
    #[arg(long)]
    is_regional_pricing_enabled: Option<bool>,
    #[arg(long)]
    is_managed_pricing_enabled: Option<bool>,
}
#[derive(Debug, Args)]
struct UpdateDeveloperArgs {
    universe_id: u64,
    product_id: u64,
    #[arg(long)]
    name: Option<String>,
    #[arg(long)]
    description: Option<String>,
    #[arg(long)]
    is_for_sale: Option<bool>,
    #[arg(long)]
    price: Option<i64>,
    #[arg(long)]
    image_file: Option<PathBuf>,
    #[arg(long, default_value = "image/png")]
    content_type: String,
    #[arg(long)]
    is_regional_pricing_enabled: Option<bool>,
    #[arg(long)]
    store_page_enabled: Option<bool>,
    #[arg(long)]
    is_managed_pricing_enabled: Option<bool>,
}
#[derive(Debug, Args)]
struct CreatePassArgs {
    universe_id: u64,
    #[arg(long)]
    name: String,
    #[arg(long)]
    description: Option<String>,
    #[arg(long)]
    is_for_sale: Option<bool>,
    #[arg(long)]
    price: Option<i64>,
    #[arg(long)]
    image_file: Option<PathBuf>,
    #[arg(long, default_value = "image/png")]
    content_type: String,
    #[arg(long)]
    is_regional_pricing_enabled: Option<bool>,
    #[arg(long)]
    is_managed_pricing_enabled: Option<bool>,
    #[arg(long)]
    is_trial_enabled: Option<bool>,
}
#[derive(Debug, Args)]
struct UpdatePassArgs {
    universe_id: u64,
    game_pass_id: u64,
    #[arg(long)]
    name: Option<String>,
    #[arg(long)]
    description: Option<String>,
    #[arg(long)]
    is_for_sale: Option<bool>,
    #[arg(long)]
    price: Option<i64>,
    #[arg(long)]
    image_file: Option<PathBuf>,
    #[arg(long)]
    file: Option<PathBuf>,
    #[arg(long, default_value = "image/png")]
    content_type: String,
    #[arg(long)]
    is_regional_pricing_enabled: Option<bool>,
    #[arg(long)]
    is_managed_pricing_enabled: Option<bool>,
    #[arg(long)]
    is_trial_enabled: Option<bool>,
}
#[derive(Debug, Args)]
struct CreateBadgeArgs {
    universe_id: u64,
    #[arg(long)]
    name: Option<String>,
    #[arg(long)]
    description: Option<String>,
    #[arg(long)]
    payment_source_type: Option<BadgePaymentSource>,
    #[arg(long)]
    expected_cost: Option<i64>,
    #[arg(long)]
    is_active: Option<bool>,
    #[arg(long)]
    image_file: Option<PathBuf>,
    #[arg(long, default_value = "image/png")]
    content_type: String,
}
#[derive(Debug, Args)]
struct BadgeIconArgs {
    badge_id: u64,
    #[arg(long)]
    image_file: PathBuf,
    #[arg(long, default_value = "image/png")]
    content_type: String,
}
impl EconomyUploads {
    pub async fn run(self) -> anyhow::Result<Option<String>> {
        let client = EconomyUploadClient::new(transport().await?);
        let value = match self.command {
            Command::CreateDeveloperProduct(args) => {
                client
                    .create_developer_product(
                        args.universe_id,
                        CreateDeveloperProductInput {
                            name: args.name,
                            description: args.description,
                            is_for_sale: args.is_for_sale,
                            price: args.price,
                            is_regional_pricing_enabled: args.is_regional_pricing_enabled,
                            is_managed_pricing_enabled: args.is_managed_pricing_enabled,
                        },
                        args.image_file.as_deref(),
                        &args.content_type,
                    )
                    .await?
            }
            Command::UpdateDeveloperProduct(args) => {
                client
                    .update_developer_product(
                        args.universe_id,
                        args.product_id,
                        UpdateDeveloperProductInput {
                            name: args.name,
                            description: args.description,
                            is_for_sale: args.is_for_sale,
                            price: args.price,
                            is_regional_pricing_enabled: args.is_regional_pricing_enabled,
                            store_page_enabled: args.store_page_enabled,
                            is_managed_pricing_enabled: args.is_managed_pricing_enabled,
                        },
                        args.image_file.as_deref(),
                        &args.content_type,
                    )
                    .await?
            }
            Command::CreateGamePass(args) => {
                client
                    .create_game_pass(
                        args.universe_id,
                        CreateGamePassInput {
                            name: args.name,
                            description: args.description,
                            is_for_sale: args.is_for_sale,
                            price: args.price,
                            is_regional_pricing_enabled: args.is_regional_pricing_enabled,
                            is_managed_pricing_enabled: args.is_managed_pricing_enabled,
                            is_trial_enabled: args.is_trial_enabled,
                        },
                        args.image_file.as_deref(),
                        &args.content_type,
                    )
                    .await?
            }
            Command::UpdateGamePass(args) => {
                client
                    .update_game_pass(
                        args.universe_id,
                        args.game_pass_id,
                        UpdateGamePassInput {
                            name: args.name,
                            description: args.description,
                            is_for_sale: args.is_for_sale,
                            price: args.price,
                            is_regional_pricing_enabled: args.is_regional_pricing_enabled,
                            is_managed_pricing_enabled: args.is_managed_pricing_enabled,
                            is_trial_enabled: args.is_trial_enabled,
                        },
                        args.image_file.as_deref(),
                        args.file.as_deref(),
                        &args.content_type,
                    )
                    .await?
            }
            Command::CreateBadge(args) => {
                client
                    .create_badge(
                        args.universe_id,
                        CreateBadgeInput {
                            name: args.name,
                            description: args.description,
                            payment_source_type: args.payment_source_type,
                            expected_cost: args.expected_cost,
                            is_active: args.is_active,
                        },
                        args.image_file.as_deref(),
                        &args.content_type,
                    )
                    .await?
            }
            Command::UpdateBadgeIcon(args) => {
                client
                    .update_badge_icon(args.badge_id, &args.image_file, &args.content_type)
                    .await?
            }
        };
        Ok(Some(serde_json::to_string_pretty(&value)?))
    }
}

#[derive(Debug, Args)]
pub struct AssetPermissionGrants {
    #[command(subcommand)]
    command: GrantCommand,
}
#[derive(Debug, Subcommand)]
enum GrantCommand {
    BatchGrant(GrantArgs),
}
#[derive(Debug, Args)]
struct GrantArgs {
    #[arg(long)]
    subject_type: Option<AssetSubjectType>,
    #[arg(long)]
    subject_id: Option<String>,
    #[arg(long)]
    action: Option<AssetGrantAction>,
    #[arg(long = "grant", value_parser = parse_grant)]
    requests: Vec<AssetGrantInput>,
    #[arg(long = "asset-id")]
    asset_ids: Vec<u64>,
    #[arg(long)]
    enable_deep_access_check: Option<bool>,
}
fn parse_grant(value: &str) -> Result<AssetGrantInput, String> {
    let mut fields = value.split(':');
    let asset_id = fields
        .next()
        .unwrap()
        .parse()
        .map_err(|_| "expected ASSET_ID[:GRANT_TO_DEPENDENCIES[:PARENT_VERSION]]")?;
    let grant_to_dependencies = fields
        .next()
        .map(str::parse)
        .transpose()
        .map_err(|_| "GRANT_TO_DEPENDENCIES must be true or false")?;
    let parent_version_number = fields
        .next()
        .map(str::parse)
        .transpose()
        .map_err(|_| "PARENT_VERSION must be an integer")?;
    if fields.next().is_some() {
        return Err("too many grant fields".into());
    }
    Ok(AssetGrantInput {
        asset_id: Some(asset_id),
        grant_to_dependencies,
        parent_version_number,
    })
}
impl AssetPermissionGrants {
    pub async fn run(self) -> anyhow::Result<Option<String>> {
        let GrantCommand::BatchGrant(args) = self.command;
        let client = AssetPermissionClient::new(transport().await?);
        let value = client
            .batch_grant(BatchGrantPermissionsInput {
                subject_type: args.subject_type,
                subject_id: args.subject_id,
                action: args.action,
                requests: (!args.requests.is_empty()).then_some(args.requests),
                asset_ids: (!args.asset_ids.is_empty()).then_some(args.asset_ids),
                enable_deep_access_check: args.enable_deep_access_check,
            })
            .await?;
        Ok(Some(serde_json::to_string_pretty(&value)?))
    }
}
