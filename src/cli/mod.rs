mod analytics_query;
mod asset_download;
mod asset_multipart_start;
mod assets_cli;
mod campaign_planning;
mod cloud;
mod datastore_cli;
mod experience_cli;
mod generated;
mod group_cli;
mod inventory_cli;
mod luau_execution_cli;
mod matchmaking_attributes;
mod messaging_cli;
mod notification_cli;
mod ordered_datastore_cli;
mod place_cli;
mod service_efficiency_typed;
mod subscription_cli;
mod universe_cli;
mod user_cli;
mod user_restriction_cli;

use clap::{Parser, Subcommand};
use generated::storage::{DatastoreV2, MemoryStore, OrderedDatastoreV2};
use inventory_cli::Inventory;
use luau_execution_cli::Luau;
use universe_cli::Universe;
use user_cli::User;
use user_restriction_cli::UserRestriction;

use self::{
    assets_cli::Assets, cloud::Coverage, datastore_cli::DataStore, experience_cli::Experience,
    group_cli::Group, messaging_cli::Messaging, notification_cli::Notification,
    ordered_datastore_cli::OrderedDataStore, place_cli::Place, subscription_cli::Subscription,
};

#[derive(Debug, Parser)]
#[clap(name = "rbxcloud", version)]
pub(crate) struct Cli {
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Command {
    /// Access the Roblox Assets API
    Assets(Assets),
    /// Manage typed service-efficiency accounts and payments
    ServicePayments(service_efficiency_typed::ServicePayments),
    /// Execute typed composed request bodies
    ComposedTools(generated::composed::ComposedTools),
    /// Execute typed translation and utility requests
    UtilityTools(generated::utility::UtilityTools),
    /// Execute typed resource actions with flat request bodies
    ActionTools(generated::actions::ActionTools),
    /// Access typed marketplace, localization, and service resources
    ResourceTools(generated::resources::ResourceTools),
    /// Access typed advertising and analytics APIs
    InsightsTools(generated::insights::InsightsTools),
    /// Access typed economy, configuration, and live operations
    LiveopsTools(generated::liveops::LiveOpsTools),
    /// Access typed creator and experience management APIs
    CreatorTools(generated::creators::CreatorTools),
    /// Manage typed matchmaking player attributes
    PlayerAttributes(matchmaking_attributes::PlayerAttributes),
    /// Run typed analytics metric and benchmark queries
    AnalyticsQuery(analytics_query::AnalyticsQuery),
    /// Create a typed advertising campaign recommendation
    CampaignPlanning(campaign_planning::CampaignRecommendation),
    /// Download authenticated binary Roblox assets
    AssetDownload(asset_download::AssetDownload),
    /// Start typed multipart asset uploads
    AssetMultipartStart(asset_multipart_start::AssetMultipartStart),
    /// Control typed asset multipart upload sessions
    AssetMultipart(generated::multipart::Assets),
    /// Access typed asset versions and quotas
    AssetTools(generated::assets::Assets),

    /// Access the Roblox Experience API
    Experience(Experience),

    /// Access the Roblox Messaging API
    Messaging(Messaging),

    /// Access the Roblox DataStore API
    Datastore(DataStore),

    /// Access the Roblox OrderedDataStore API
    OrderedDatastore(OrderedDataStore),

    /// Access the Roblox user inventory API
    Inventory(Inventory),

    /// Access the Roblox Group API
    Group(Group),

    Luau(Luau),

    /// Access the Roblox Subscription API
    Subscription(Subscription),

    /// Access the Roblox Notification API
    Notification(Notification),

    /// Access the Roblox Place API
    Place(Place),

    /// Access the Roblox Universe API
    Universe(Universe),

    /// Access the Roblox User API
    User(User),

    /// Access to the Roblox User Restriction API
    UserRestriction(UserRestriction),

    /// Access typed Open Cloud v2 DataStores
    DatastoreV2(DatastoreV2),
    /// Access typed Open Cloud Memory Store
    MemoryStore(MemoryStore),
    /// Access typed Open Cloud v2 ordered DataStores
    OrderedDatastoreV2(OrderedDatastoreV2),

    /// Show Open Cloud API coverage status by family
    Coverage(Coverage),
}

impl Cli {
    pub(crate) async fn run(self) -> anyhow::Result<Option<String>> {
        match self.command {
            Command::Assets(command) => command.run().await,
            Command::AssetTools(command) => command.run().await,
            Command::AssetMultipart(command) => command.run().await,
            Command::AssetMultipartStart(command) => command.run().await,
            Command::AssetDownload(command) => command.run().await,
            Command::CampaignPlanning(command) => command.run().await,
            Command::AnalyticsQuery(command) => command.run().await,
            Command::PlayerAttributes(command) => command.run().await,
            Command::CreatorTools(command) => command.run().await,
            Command::LiveopsTools(command) => command.run().await,
            Command::InsightsTools(command) => command.run().await,
            Command::ResourceTools(command) => command.run().await,
            Command::ActionTools(command) => command.run().await,
            Command::UtilityTools(command) => command.run().await,
            Command::ComposedTools(command) => command.run().await,
            Command::ServicePayments(command) => command.run().await,
            Command::Experience(command) => command.run().await,
            Command::Messaging(command) => command.run().await,
            Command::Datastore(command) => command.run().await,
            Command::OrderedDatastore(command) => command.run().await,
            Command::Group(command) => command.run().await,
            Command::Inventory(command) => command.run().await,
            Command::Luau(command) => command.run().await,
            Command::Subscription(command) => command.run().await,
            Command::Notification(command) => command.run().await,
            Command::Place(command) => command.run().await,
            Command::Universe(command) => command.run().await,
            Command::User(command) => command.run().await,
            Command::UserRestriction(command) => command.run().await,
            Command::DatastoreV2(command) => command.run().await,
            Command::MemoryStore(command) => command.run().await,
            Command::OrderedDatastoreV2(command) => command.run().await,
            Command::Coverage(command) => command.run(),
        }
    }
}
