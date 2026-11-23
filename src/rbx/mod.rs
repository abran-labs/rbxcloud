//! Access into Roblox APIs.
//!
//! Most usage should go through the `RbxCloud` struct.
//!
//! [`client`] and [`credentials`] are the shared transport that new typed
//! API families are built on. The `v1` and `v2` clients do not use it yet.
pub mod analytics_queries;
pub mod asset_delivery;
pub mod asset_multipart;
pub mod campaign_planning;
pub mod client;
pub mod config_publishing;
pub mod credentials;
pub mod error;
pub mod experiment_management;
#[path = "generated/mod.rs"]
pub mod generated;
pub mod matchmaking_attributes;
pub mod service_efficiency;
pub mod types;
pub(crate) mod util;
pub mod v1;
pub mod v2;

pub mod campaign_management;

pub mod matchmaking_configuration;

pub mod game_localization;
pub mod localization_table_contents;

pub mod analytics_alerts;
pub mod analytics_dashboards;
pub mod analytics_log_regexes;

pub mod avatar_bundle_context;
pub mod creator_store_management;
pub mod item_folders;

pub mod asset_permission_grants;
pub mod economy_uploads;

pub mod collectible_management;
pub mod place_instance_management;
pub mod server_restarts;
pub mod service_efficiency_configuration;
pub mod speech_assets;
pub mod virtual_event_management;
