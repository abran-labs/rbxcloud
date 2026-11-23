use clap::{Args, Subcommand, ValueEnum};
use rbxcloud::rbx::{
    client::{AsyncClient, ClientPolicy},
    collectible_management::*,
    credentials::Credentials,
    place_instance_management::*,
    server_restarts::*,
    service_efficiency_configuration::*,
    speech_assets::*,
    virtual_event_management::*,
};
use std::collections::BTreeMap;

async fn transport() -> anyhow::Result<AsyncClient> {
    Ok(AsyncClient::new(Credentials::from_process()?, ClientPolicy::default()).await?)
}

#[derive(Debug, Args)]
pub struct CollectibleManagement {
    #[command(subcommand)]
    command: CollectibleCommand,
}
#[derive(Debug, Subcommand)]
enum CollectibleCommand {
    Publish(PublishCollectibleArgs),
    BulkUpdate(BulkCollectibleArgs),
    Update(UpdateCollectibleArgs),
}
#[derive(Debug, Args, Default)]
struct SaleLocationArgs {
    #[arg(long)]
    sale_location_type: Option<CollectibleSaleLocationType>,
    #[arg(long = "sale-place-id")]
    places: Vec<u64>,
}
impl SaleLocationArgs {
    fn build(self) -> Option<CollectibleSaleLocationInput> {
        (self.sale_location_type.is_some() || !self.places.is_empty()).then_some(
            CollectibleSaleLocationInput {
                sale_location_type: self.sale_location_type,
                places: (!self.places.is_empty()).then_some(self.places),
            },
        )
    }
}
#[derive(Debug, Args)]
struct PublishCollectibleArgs {
    #[arg(long)]
    is_rental_opt_in: Option<bool>,
    #[arg(long)]
    idempotency_token: Option<String>,
    #[arg(long)]
    target_id: Option<u64>,
    #[arg(long)]
    target_type: Option<CollectibleTargetType>,
    #[arg(long)]
    publishing_type: Option<CollectiblePublishingType>,
    #[arg(long)]
    agreed_publishing_fee: Option<u64>,
    #[arg(long)]
    creator_user_id: Option<u64>,
    #[arg(long)]
    creator_group_id: Option<u64>,
    #[arg(long)]
    publisher_user_id: Option<u64>,
    #[arg(long)]
    quantity: Option<u32>,
    #[arg(long)]
    quantity_limit_per_user: Option<u32>,
    #[arg(long)]
    resale_restriction: Option<CollectibleResaleRestriction>,
    #[arg(long)]
    price_in_robux: Option<u64>,
    #[arg(long)]
    price_offset: Option<u64>,
    #[arg(long)]
    opt_out_from_regional_pricing: Option<bool>,
    #[arg(long)]
    is_free: Option<bool>,
    #[command(flatten)]
    sale_location: SaleLocationArgs,
    #[arg(long)]
    name: Option<String>,
    #[arg(long)]
    description: Option<String>,
    #[arg(long)]
    on_sale_time: Option<String>,
    #[arg(long)]
    off_sale_time: Option<String>,
    #[arg(long)]
    agreement_id: Option<String>,
}
#[derive(Debug, Args)]
struct BulkCollectibleArgs {
    #[arg(long)]
    idempotency_key: Option<String>,
    #[arg(long)]
    group_id: Option<u64>,
    #[arg(long = "asset-type")]
    asset_types: Vec<u32>,
    #[arg(long = "target", value_parser = parse_collectible_target)]
    targets: Vec<CollectibleTargetInput>,
    #[arg(long)]
    rental_opt_in: Option<bool>,
}
fn parse_collectible_target(value: &str) -> Result<CollectibleTargetInput, String> {
    let (kind, id) = value.split_once(':').ok_or("expected TYPE:ID")?;
    let target_type = match kind {
        "asset" => CollectibleTargetType::Asset,
        "bundle" => CollectibleTargetType::Bundle,
        "iec-token" => CollectibleTargetType::IecToken,
        "look" => CollectibleTargetType::Look,
        _ => return Err("expected asset, bundle, iec-token, or look".into()),
    };
    Ok(CollectibleTargetInput {
        target_type: Some(target_type),
        target_id: Some(id.into()),
    })
}
#[derive(Debug, Args)]
struct UpdateCollectibleArgs {
    collectible_item_id: String,
    #[command(flatten)]
    sale_location: SaleLocationArgs,
    #[arg(long)]
    is_rental_opt_in: Option<bool>,
    #[arg(long)]
    sale_status: Option<CollectibleSaleStatus>,
    #[arg(long)]
    quantity_limit_per_user: Option<u32>,
    #[arg(long)]
    resale_restriction: Option<CollectibleResaleRestriction>,
    #[arg(long)]
    price_in_robux: Option<u64>,
    #[arg(long)]
    price_offset: Option<u64>,
    #[arg(long)]
    opt_out_from_regional_pricing: Option<bool>,
    #[arg(long)]
    is_free: Option<bool>,
    #[arg(long)]
    on_sale_time: Option<String>,
    #[arg(long)]
    off_sale_time: Option<String>,
}
impl CollectibleManagement {
    pub async fn run(self) -> anyhow::Result<Option<String>> {
        let client = CollectibleClient::new(transport().await?);
        let value = match self.command {
            CollectibleCommand::Publish(args) => {
                client
                    .publish(PublishCollectibleInput {
                        is_rental_opt_in: args.is_rental_opt_in,
                        idempotency_token: args.idempotency_token,
                        target_id: args.target_id,
                        target_type: args.target_type,
                        publishing_type: args.publishing_type,
                        agreed_publishing_fee: args.agreed_publishing_fee,
                        creator_user_id: args.creator_user_id,
                        creator_group_id: args.creator_group_id,
                        publisher_user_id: args.publisher_user_id,
                        quantity: args.quantity,
                        quantity_limit_per_user: args.quantity_limit_per_user,
                        resale_restriction: args.resale_restriction,
                        price_in_robux: args.price_in_robux,
                        price_offset: args.price_offset,
                        opt_out_from_regional_pricing: args.opt_out_from_regional_pricing,
                        is_free: args.is_free,
                        sale_location_configuration: args.sale_location.build(),
                        name: args.name,
                        description: args.description,
                        on_sale_time: args.on_sale_time,
                        off_sale_time: args.off_sale_time,
                        agreement_id: args.agreement_id,
                    })
                    .await?
            }
            CollectibleCommand::BulkUpdate(args) => {
                client
                    .bulk_update(BulkUpdateCollectiblesInput {
                        idempotency_key: args.idempotency_key,
                        group_id: args.group_id,
                        update_targets: (!args.asset_types.is_empty() || !args.targets.is_empty())
                            .then_some(BulkUpdateTargetsInput {
                                asset_type: (!args.asset_types.is_empty())
                                    .then_some(args.asset_types),
                                target: (!args.targets.is_empty()).then_some(args.targets),
                            }),
                        update_payload: args.rental_opt_in.map(|rental_opt_in| {
                            BulkUpdatePayloadInput {
                                rental_opt_in: Some(rental_opt_in),
                            }
                        }),
                    })
                    .await?
            }
            CollectibleCommand::Update(args) => {
                client
                    .update(
                        &args.collectible_item_id,
                        UpdateCollectibleInput {
                            sale_location_configuration: args.sale_location.build(),
                            is_rental_opt_in: args.is_rental_opt_in,
                            sale_status: args.sale_status,
                            quantity_limit_per_user: args.quantity_limit_per_user,
                            resale_restriction: args.resale_restriction,
                            price_in_robux: args.price_in_robux,
                            price_offset: args.price_offset,
                            opt_out_from_regional_pricing: args.opt_out_from_regional_pricing,
                            is_free: args.is_free,
                            on_sale_time: args.on_sale_time,
                            off_sale_time: args.off_sale_time,
                        },
                    )
                    .await?
            }
        };
        Ok(Some(serde_json::to_string_pretty(&value)?))
    }
}

#[derive(Debug, Args)]
pub struct PlaceInstanceManagement {
    #[command(subcommand)]
    command: PlaceInstanceCommand,
}
#[derive(Debug, Subcommand)]
enum PlaceInstanceCommand {
    Update(PlaceInstanceArgs),
}
#[derive(Debug, Args)]
struct PlaceInstanceArgs {
    universe_id: u64,
    place_id: u64,
    instance_id: String,
    #[arg(long)]
    path: Option<String>,
    #[arg(long)]
    update_mask: Option<String>,
    #[arg(long)]
    engine_id: Option<String>,
    #[arg(long)]
    engine_parent: Option<String>,
    #[arg(long)]
    engine_name: Option<String>,
    #[arg(long)]
    folder: bool,
    #[arg(long)]
    local_script_enabled: Option<bool>,
    #[arg(long)]
    local_script_run_context: Option<EngineRunContext>,
    #[arg(long)]
    local_script_source: Option<String>,
    #[arg(long)]
    module_script_source: Option<String>,
    #[arg(long)]
    script_enabled: Option<bool>,
    #[arg(long)]
    script_run_context: Option<EngineRunContext>,
    #[arg(long)]
    script_source: Option<String>,
}
impl PlaceInstanceManagement {
    pub async fn run(self) -> anyhow::Result<Option<String>> {
        let PlaceInstanceCommand::Update(args) = self.command;
        let local = (args.local_script_enabled.is_some()
            || args.local_script_run_context.is_some()
            || args.local_script_source.is_some())
        .then_some(EngineScriptInput {
            enabled: args.local_script_enabled,
            run_context: args.local_script_run_context,
            source: args.local_script_source,
        });
        let module = args
            .module_script_source
            .map(|source| EngineModuleScriptInput {
                source: Some(source),
            });
        let script = (args.script_enabled.is_some()
            || args.script_run_context.is_some()
            || args.script_source.is_some())
        .then_some(EngineScriptInput {
            enabled: args.script_enabled,
            run_context: args.script_run_context,
            source: args.script_source,
        });
        let details = (args.folder || local.is_some() || module.is_some() || script.is_some())
            .then_some(EngineInstanceDetailsInput {
                folder: args.folder.then_some(EngineFolderInput::default()),
                local_script: local,
                module_script: module,
                script,
            });
        let engine = (args.engine_id.is_some()
            || args.engine_parent.is_some()
            || args.engine_name.is_some()
            || details.is_some())
        .then_some(EngineInstanceInput {
            id: args.engine_id,
            parent: args.engine_parent,
            name: args.engine_name,
            details,
        });
        let client = PlaceInstanceClient::new(transport().await?);
        let value = client
            .update(
                args.universe_id,
                args.place_id,
                &args.instance_id,
                UpdatePlaceInstanceInput {
                    path: args.path,
                    engine_instance: engine,
                },
                args.update_mask,
            )
            .await?;
        Ok(Some(serde_json::to_string_pretty(&value)?))
    }
}

#[derive(Debug, Args)]
pub struct SpeechAssets {
    #[command(subcommand)]
    command: SpeechCommand,
}
#[derive(Debug, Subcommand)]
enum SpeechCommand {
    Generate(SpeechArgs),
}
#[derive(Debug, Args)]
struct SpeechArgs {
    universe_id: u64,
    #[arg(long)]
    text: String,
    #[arg(long)]
    voice_id: Option<String>,
    #[arg(long, allow_negative_numbers = true)]
    pitch: Option<f64>,
    #[arg(long, allow_negative_numbers = true)]
    speed: Option<f64>,
}
impl SpeechAssets {
    pub async fn run(self) -> anyhow::Result<Option<String>> {
        let SpeechCommand::Generate(args) = self.command;
        let style = (args.voice_id.is_some() || args.pitch.is_some() || args.speed.is_some())
            .then_some(SpeechStyleInput {
                voice_id: args.voice_id,
                pitch: args.pitch,
                speed: args.speed,
            });
        let client = SpeechAssetClient::new(transport().await?);
        let value = client
            .generate(
                args.universe_id,
                GenerateSpeechInput {
                    text: args.text,
                    speech_style: style,
                },
            )
            .await?;
        Ok(Some(serde_json::to_string_pretty(&value)?))
    }
}

#[derive(Debug, Args)]
pub struct ServerRestarts {
    #[command(subcommand)]
    command: RestartCommand,
}
#[derive(Debug, Subcommand)]
enum RestartCommand {
    Restart(RestartArgs),
}
#[derive(Debug, Args)]
struct RestartArgs {
    universe_id: u64,
    #[arg(long)]
    bleed_off_duration_minutes: Option<u32>,
    #[arg(long)]
    places_json: Option<String>,
    #[arg(long)]
    attributes_json: Option<String>,
}
impl ServerRestarts {
    pub async fn run(self) -> anyhow::Result<Option<String>> {
        let RestartCommand::Restart(args) = self.command;
        let places = args
            .places_json
            .as_deref()
            .map(serde_json::from_str::<BTreeMap<String, PlaceVersionFilterInput>>)
            .transpose()?;
        let attributes = args
            .attributes_json
            .as_deref()
            .map(serde_json::from_str)
            .transpose()?;
        let client = ServerRestartClient::new(transport().await?);
        let value = client
            .restart(
                args.universe_id,
                LaunchRestartInput {
                    bleed_off_duration_minutes: args.bleed_off_duration_minutes,
                    places,
                    attributes,
                },
            )
            .await?;
        Ok(Some(serde_json::to_string_pretty(&value)?))
    }
}

#[derive(Debug, Args)]
pub struct ServiceEfficiencyConfiguration {
    #[command(subcommand)]
    command: ServiceCommand,
}
#[derive(Debug, Subcommand)]
enum ServiceCommand {
    Update(ServiceArgs),
}
#[derive(Debug, Args)]
struct ServiceArgs {
    universe_id: u64,
    #[arg(long)]
    service_configurations_json: String,
}
impl ServiceEfficiencyConfiguration {
    pub async fn run(self) -> anyhow::Result<Option<String>> {
        let ServiceCommand::Update(args) = self.command;
        let configurations = serde_json::from_str(&args.service_configurations_json)?;
        let client = ServiceEfficiencyClient::new(transport().await?);
        let value = client.update(args.universe_id, configurations).await?;
        Ok(Some(serde_json::to_string_pretty(&value)?))
    }
}

#[derive(Debug, Args)]
pub struct VirtualEventManagement {
    #[command(subcommand)]
    command: EventCommand,
}
#[derive(Debug, Subcommand)]
enum EventCommand {
    Create(CreateEventArgs),
    Update(UpdateEventArgs),
}
#[derive(Debug, Args, Default)]
struct EventFields {
    #[arg(long)]
    title: Option<String>,
    #[arg(long)]
    subtitle: Option<String>,
    #[arg(long)]
    description: Option<String>,
    #[arg(long)]
    start_time: Option<String>,
    #[arg(long)]
    end_time: Option<String>,
    #[arg(long)]
    visibility: Option<EventVisibilityInput>,
    #[arg(long)]
    place_id: Option<u64>,
    #[arg(long = "category", value_parser = parse_event_category)]
    categories: Vec<EventRankedCategoryInput>,
    #[arg(long = "thumbnail", value_parser = parse_event_media)]
    thumbnails: Vec<EventMediaInput>,
    #[arg(long)]
    featuring_status: Option<EventFeaturingStatus>,
    #[arg(long)]
    tagline: Option<String>,
}
#[derive(Debug, Args)]
struct CreateEventArgs {
    universe_id: u64,
    #[arg(long)]
    group_id: Option<u64>,
    #[command(flatten)]
    fields: EventFields,
}
#[derive(Debug, Args)]
struct UpdateEventArgs {
    event_id: String,
    #[command(flatten)]
    fields: EventFields,
}
fn parse_event_category(value: &str) -> Result<EventRankedCategoryInput, String> {
    let (category, rank) = value.split_once(':').ok_or("expected CATEGORY:RANK")?;
    Ok(EventRankedCategoryInput {
        category: Some(EventCategoryInput::from_str(category, true)?),
        rank: Some(rank.parse().map_err(|_| "RANK must be an integer")?),
    })
}
fn parse_event_media(value: &str) -> Result<EventMediaInput, String> {
    let (id, rank) = value.split_once(':').ok_or("expected MEDIA_ID:RANK")?;
    Ok(EventMediaInput {
        media_id: Some(id.parse().map_err(|_| "MEDIA_ID must be an integer")?),
        rank: Some(rank.parse().map_err(|_| "RANK must be an integer")?),
    })
}
impl VirtualEventManagement {
    pub async fn run(self) -> anyhow::Result<Option<String>> {
        let client = VirtualEventClient::new(transport().await?);
        let value = match self.command {
            EventCommand::Create(args) => {
                let f = args.fields;
                client
                    .create(
                        args.universe_id,
                        CreateVirtualEventInput {
                            title: f.title,
                            subtitle: f.subtitle,
                            description: f.description,
                            start_time: f.start_time,
                            end_time: f.end_time,
                            visibility: f.visibility,
                            group_id: args.group_id,
                            place_id: f.place_id,
                            categories: (!f.categories.is_empty()).then_some(f.categories),
                            thumbnails: (!f.thumbnails.is_empty()).then_some(f.thumbnails),
                            featuring_status: f.featuring_status,
                            tagline: f.tagline,
                        },
                    )
                    .await?
            }
            EventCommand::Update(args) => {
                let f = args.fields;
                client
                    .update(
                        &args.event_id,
                        UpdateVirtualEventInput {
                            title: f.title,
                            subtitle: f.subtitle,
                            description: f.description,
                            start_time: f.start_time,
                            end_time: f.end_time,
                            visibility: f.visibility,
                            place_id: f.place_id,
                            categories: (!f.categories.is_empty()).then_some(f.categories),
                            thumbnails: (!f.thumbnails.is_empty()).then_some(f.thumbnails),
                            featuring_status: f.featuring_status,
                            tagline: f.tagline,
                        },
                    )
                    .await?
            }
        };
        Ok(Some(serde_json::to_string_pretty(&value)?))
    }
}
