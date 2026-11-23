use clap::{Args, Subcommand};
use rbxcloud::rbx::{
    avatar_bundle_context::*,
    client::{AsyncClient, ClientPolicy},
    creator_store_management::*,
    credentials::Credentials,
    item_folders::*,
};
use std::collections::BTreeMap;

async fn transport() -> anyhow::Result<AsyncClient> {
    Ok(AsyncClient::new(Credentials::from_process()?, ClientPolicy::default()).await?)
}

#[derive(Debug, Args)]
pub struct CreatorStoreProducts {
    #[command(subcommand)]
    command: ProductCommand,
}
#[derive(Debug, Subcommand)]
enum ProductCommand {
    CreateProduct(CreateProductArgs),
    UpdateProduct(UpdateProductArgs),
}
#[derive(Debug, Args, Default)]
struct BasePriceArgs {
    #[arg(long, requires_all = ["base_price_significand", "base_price_exponent"])]
    base_price_currency_code: Option<String>,
    #[arg(long, requires_all = ["base_price_currency_code", "base_price_exponent"])]
    base_price_significand: Option<i64>,
    #[arg(long, requires_all = ["base_price_currency_code", "base_price_significand"])]
    base_price_exponent: Option<i32>,
}
#[derive(Debug, Args, Default)]
struct PurchasePriceArgs {
    #[arg(long, requires_all = ["purchase_price_significand", "purchase_price_exponent"])]
    purchase_price_currency_code: Option<String>,
    #[arg(long, requires_all = ["purchase_price_significand", "purchase_price_exponent"])]
    purchase_price_significand: Option<i64>,
    #[arg(long, requires_all = ["purchase_price_currency_code", "purchase_price_significand"])]
    purchase_price_exponent: Option<i32>,
}
fn money(
    currency: Option<String>,
    significand: Option<i64>,
    exponent: Option<i32>,
) -> Option<MoneyInput> {
    currency.map(|currency_code| MoneyInput {
        currency_code: Some(currency_code),
        quantity: Some(DecimalInput {
            significand,
            exponent,
        }),
    })
}
#[derive(Debug, Args)]
struct CreateProductArgs {
    #[arg(long)]
    path: Option<String>,
    #[command(flatten)]
    base: BasePriceArgs,
    #[command(flatten)]
    purchase: PurchasePriceArgs,
    #[arg(long)]
    published: Option<bool>,
    #[arg(long)]
    model_asset_id: Option<String>,
    #[arg(long)]
    plugin_asset_id: Option<String>,
    #[arg(long)]
    audio_asset_id: Option<String>,
    #[arg(long)]
    decal_asset_id: Option<String>,
    #[arg(long)]
    mesh_part_asset_id: Option<String>,
    #[arg(long)]
    video_asset_id: Option<String>,
    #[arg(long)]
    font_family_asset_id: Option<String>,
}
#[derive(Debug, Args)]
struct UpdateProductArgs {
    creator_store_product_id: String,
    #[arg(long)]
    path: Option<String>,
    #[command(flatten)]
    base: BasePriceArgs,
    #[command(flatten)]
    purchase: PurchasePriceArgs,
    #[arg(long)]
    published: Option<bool>,
    #[arg(long)]
    update_mask: Option<String>,
    #[arg(long)]
    allow_missing: Option<bool>,
}
impl CreatorStoreProducts {
    pub async fn run(self) -> anyhow::Result<Option<String>> {
        let client = CreatorStoreProductClient::new(transport().await?);
        let value = match self.command {
            ProductCommand::CreateProduct(args) => {
                client
                    .create_product(CreateCreatorStoreProductInput {
                        path: args.path,
                        base_price: money(
                            args.base.base_price_currency_code,
                            args.base.base_price_significand,
                            args.base.base_price_exponent,
                        ),
                        purchase_price: money(
                            args.purchase.purchase_price_currency_code,
                            args.purchase.purchase_price_significand,
                            args.purchase.purchase_price_exponent,
                        ),
                        published: args.published,
                        model_asset_id: args.model_asset_id,
                        plugin_asset_id: args.plugin_asset_id,
                        audio_asset_id: args.audio_asset_id,
                        decal_asset_id: args.decal_asset_id,
                        mesh_part_asset_id: args.mesh_part_asset_id,
                        video_asset_id: args.video_asset_id,
                        font_family_asset_id: args.font_family_asset_id,
                    })
                    .await?
            }
            ProductCommand::UpdateProduct(args) => {
                client
                    .update_product(
                        args.creator_store_product_id,
                        UpdateCreatorStoreProductInput {
                            path: args.path,
                            base_price: money(
                                args.base.base_price_currency_code,
                                args.base.base_price_significand,
                                args.base.base_price_exponent,
                            ),
                            purchase_price: money(
                                args.purchase.purchase_price_currency_code,
                                args.purchase.purchase_price_significand,
                                args.purchase.purchase_price_exponent,
                            ),
                            published: args.published,
                        },
                        args.update_mask,
                        args.allow_missing,
                    )
                    .await?
            }
        };
        Ok(Some(serde_json::to_string_pretty(&value)?))
    }
}

#[derive(Debug, Args)]
pub struct ToolboxAssets {
    #[command(subcommand)]
    command: ToolboxCommand,
}
#[derive(Debug, Subcommand)]
enum ToolboxCommand {
    BulkDeleteSaves(BulkDeleteArgs),
    SearchAssets(SearchArgs),
}
#[derive(Debug, Args)]
struct BulkDeleteArgs {
    #[arg(long = "target", value_parser = parse_delete_target)]
    targets: Vec<DeleteSaveInput>,
}
fn parse_delete_target(value: &str) -> Result<DeleteSaveInput, String> {
    let (kind, id) = value.split_once(':').ok_or("expected TYPE:ID")?;
    let target_type = match kind {
        "Audio" => SearchCategoryType::Audio,
        "Place" => SearchCategoryType::Place,
        "Model" => SearchCategoryType::Model,
        "Decal" => SearchCategoryType::Decal,
        "Plugin" => SearchCategoryType::Plugin,
        "MeshPart" => SearchCategoryType::MeshPart,
        "Video" => SearchCategoryType::Video,
        "FontFamily" => SearchCategoryType::FontFamily,
        _ => return Err("unknown target type".into()),
    };
    Ok(DeleteSaveInput {
        target_type,
        target_id: id.parse().map_err(|_| "ID must be an integer")?,
        collection_name: None,
    })
}
#[derive(Debug, Args, Default)]
struct SearchArgs {
    #[arg(long)]
    search_category_type: Option<SearchCategoryType>,
    #[arg(long, conflicts_with = "image")]
    query: Option<String>,
    #[arg(long, conflicts_with = "query")]
    image: Option<String>,
    #[arg(long, conflicts_with = "group_id")]
    user_id: Option<u64>,
    #[arg(long, conflicts_with = "user_id")]
    group_id: Option<u64>,
    #[arg(long, conflicts_with = "page_number")]
    page_token: Option<String>,
    #[arg(long, conflicts_with = "page_token")]
    page_number: Option<u32>,
    #[arg(long)]
    max_page_size: Option<u32>,
    #[arg(long)]
    sort_direction: Option<CreatorStoreSortDirection>,
    #[arg(long)]
    sort_category: Option<CreatorStoreSortCategory>,
    #[arg(long)]
    include_only_verified_creators: Option<bool>,
    #[arg(long)]
    min_price_cents: Option<u64>,
    #[arg(long)]
    max_price_cents: Option<u64>,
    #[arg(long, value_delimiter = ',')]
    facets: Vec<String>,
    #[arg(long, value_delimiter = ',')]
    tags: Vec<String>,
    #[arg(long)]
    category_path: Option<String>,
    #[arg(long)]
    swimlane: Option<String>,
    #[arg(long)]
    search_view: Option<CreatorStoreSearchView>,
    #[arg(long, value_delimiter = ',')]
    audio_types: Vec<SearchAudioType>,
    #[arg(long)]
    min_duration_seconds: Option<u32>,
    #[arg(long)]
    max_duration_seconds: Option<u32>,
    #[arg(long)]
    artist: Option<String>,
    #[arg(long)]
    album: Option<String>,
    #[arg(long)]
    music_chart_type: Option<MusicChartType>,
    #[arg(long, value_delimiter = ',')]
    included_sub_types: Vec<ModelSubType>,
    #[arg(long, value_delimiter = ',')]
    excluded_sub_types: Vec<ModelSubType>,
    #[arg(long, value_delimiter = ',')]
    included_instance_types: Vec<ModelInstanceType>,
}
impl ToolboxAssets {
    pub async fn run(self) -> anyhow::Result<Option<String>> {
        let client = ToolboxClient::new(transport().await?);
        let value = match self.command {
            ToolboxCommand::BulkDeleteSaves(args) => client.bulk_delete_saves(args.targets).await?,
            ToolboxCommand::SearchAssets(args) => {
                let audio = (!args.audio_types.is_empty()
                    || args.min_duration_seconds.is_some()
                    || args.max_duration_seconds.is_some()
                    || args.artist.is_some()
                    || args.album.is_some()
                    || args.music_chart_type.is_some())
                .then_some(AudioSearchFiltersInput {
                    audio_types: (!args.audio_types.is_empty()).then_some(args.audio_types),
                    min_duration_seconds: args.min_duration_seconds,
                    max_duration_seconds: args.max_duration_seconds,
                    artist: args.artist,
                    album: args.album,
                    music_chart_type: args.music_chart_type,
                });
                let model = (!args.included_sub_types.is_empty()
                    || !args.excluded_sub_types.is_empty()
                    || !args.included_instance_types.is_empty())
                .then_some(ModelSearchFiltersInput {
                    included_sub_types: (!args.included_sub_types.is_empty())
                        .then_some(args.included_sub_types),
                    excluded_sub_types: (!args.excluded_sub_types.is_empty())
                        .then_some(args.excluded_sub_types),
                    included_instance_types: (!args.included_instance_types.is_empty())
                        .then_some(args.included_instance_types),
                });
                client
                    .search_assets(SearchCreatorStoreAssetsInput {
                        search_category_type: args.search_category_type,
                        query: args.query,
                        image: args.image,
                        user_id: args.user_id,
                        group_id: args.group_id,
                        page_token: args.page_token,
                        page_number: args.page_number,
                        max_page_size: args.max_page_size,
                        sort_direction: args.sort_direction,
                        sort_category: args.sort_category,
                        include_only_verified_creators: args.include_only_verified_creators,
                        min_price_cents: args.min_price_cents,
                        max_price_cents: args.max_price_cents,
                        facets: (!args.facets.is_empty()).then_some(args.facets),
                        tags: (!args.tags.is_empty()).then_some(args.tags),
                        category_path: args.category_path,
                        swimlane: args.swimlane,
                        search_view: args.search_view,
                        audio_search_filters: audio,
                        model_search_filters: model,
                    })
                    .await?
            }
        };
        Ok(Some(serde_json::to_string_pretty(&value)?))
    }
}

#[derive(Debug, Args)]
pub struct AvatarBundleContext {
    #[command(subcommand)]
    command: BundleCommand,
}
#[derive(Debug, Subcommand)]
enum BundleCommand {
    CreateContext(BundleArgs),
}
#[derive(Debug, Args)]
struct BundleArgs {
    #[arg(long = "asset-quantity", value_parser = parse_quantity)]
    asset_quantities: Vec<(String, i32)>,
    #[arg(long)]
    bundle_description: Option<String>,
    #[arg(long)]
    bundle_name: Option<String>,
    #[arg(long)]
    bundle_type: Option<BundleType>,
    #[arg(long, allow_negative_numbers = true)]
    height: Option<f64>,
    #[arg(long, allow_negative_numbers = true)]
    width: Option<f64>,
    #[arg(long, allow_negative_numbers = true)]
    head: Option<f64>,
    #[arg(long, allow_negative_numbers = true)]
    body_type: Option<f64>,
    #[arg(long, allow_negative_numbers = true)]
    proportion: Option<f64>,
    #[arg(long)]
    head_color3: Option<String>,
    #[arg(long)]
    torso_color3: Option<String>,
    #[arg(long)]
    left_arm_color3: Option<String>,
    #[arg(long)]
    right_arm_color3: Option<String>,
    #[arg(long)]
    left_leg_color3: Option<String>,
    #[arg(long)]
    right_leg_color3: Option<String>,
    #[arg(long)]
    group_id: Option<u64>,
    #[arg(long)]
    is_emissive: Option<bool>,
}
fn parse_quantity(value: &str) -> Result<(String, i32), String> {
    let (name, count) = value.split_once('=').ok_or("expected ASSET_TYPE=COUNT")?;
    Ok((
        name.into(),
        count.parse().map_err(|_| "COUNT must be an integer")?,
    ))
}
impl AvatarBundleContext {
    pub async fn run(self) -> anyhow::Result<Option<String>> {
        let BundleCommand::CreateContext(args) = self.command;
        let scale = (args.height.is_some()
            || args.width.is_some()
            || args.head.is_some()
            || args.body_type.is_some()
            || args.proportion.is_some())
        .then_some(BodyScaleInput {
            height: args.height,
            width: args.width,
            head: args.head,
            body_type: args.body_type,
            proportion: args.proportion,
        });
        let colors = (args.head_color3.is_some()
            || args.torso_color3.is_some()
            || args.left_arm_color3.is_some()
            || args.right_arm_color3.is_some()
            || args.left_leg_color3.is_some()
            || args.right_leg_color3.is_some())
        .then_some(BodyColorSetInput {
            head_color3: args.head_color3,
            torso_color3: args.torso_color3,
            left_arm_color3: args.left_arm_color3,
            right_arm_color3: args.right_arm_color3,
            left_leg_color3: args.left_leg_color3,
            right_leg_color3: args.right_leg_color3,
        });
        let client = AvatarBundleContextClient::new(transport().await?);
        let value = client
            .create_context(CreateBundleContextInput {
                asset_quantities: (!args.asset_quantities.is_empty()).then(|| {
                    args.asset_quantities
                        .into_iter()
                        .collect::<BTreeMap<_, _>>()
                }),
                bundle_description: args.bundle_description,
                bundle_name: args.bundle_name,
                bundle_type: args.bundle_type,
                body_scale: scale,
                body_color_set: colors,
                group_id: args.group_id,
                is_emissive: args.is_emissive,
            })
            .await?;
        Ok(Some(serde_json::to_string_pretty(&value)?))
    }
}

#[derive(Debug, Args)]
pub struct ItemFolders {
    #[command(subcommand)]
    command: FolderCommand,
}
#[derive(Debug, Subcommand)]
enum FolderCommand {
    AddItems(FolderArgs),
    DeleteItems(FolderArgs),
}
#[derive(Debug, Args)]
struct FolderArgs {
    folder_id: String,
    #[arg(long = "item", value_parser = parse_folder_item)]
    items: Vec<FolderItemInput>,
}
fn parse_folder_item(value: &str) -> Result<FolderItemInput, String> {
    let (kind, id) = value.split_once(':').ok_or("expected TYPE:ID")?;
    let item_type = match kind {
        "Unknown" => FolderItemType::Unknown,
        "Asset" => FolderItemType::Asset,
        "Bundle" => FolderItemType::Bundle,
        "IECToken" => FolderItemType::IECToken,
        "Folder" => FolderItemType::Folder,
        _ => return Err("unknown item type".into()),
    };
    Ok(FolderItemInput {
        id: Some(id.into()),
        item_type: Some(item_type),
    })
}
impl ItemFolders {
    pub async fn run(self) -> anyhow::Result<Option<String>> {
        let client = ItemFolderClient::new(transport().await?);
        let value = match self.command {
            FolderCommand::AddItems(args) => client.add_items(args.folder_id, args.items).await?,
            FolderCommand::DeleteItems(args) => {
                client.delete_items(args.folder_id, args.items).await?
            }
        };
        Ok(Some(serde_json::to_string_pretty(&value)?))
    }
}
