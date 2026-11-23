//! Typed collectible publishing, bulk updates, and single-item updates.

use super::client::{AsyncClient, Error, Operation, Request, Server};
use serde::{Serialize, Serializer};
use serde_json::Value;

macro_rules! numeric_enum {
    ($name:ident { $($variant:ident = $value:literal),+ $(,)? }) => {
        #[derive(Debug, Clone, Copy, clap::ValueEnum)]
        pub enum $name { $($variant = $value),+ }
        impl Serialize for $name {
            fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> { serializer.serialize_i64(*self as i64) }
        }
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{}", *self as i64) }
        }
    };
}
numeric_enum!(CollectibleTargetType { Asset = 0, Bundle = 1, IecToken = 2, Look = 3 });
numeric_enum!(CollectiblePublishingType { Invalid = 0, Limited = 1, NonLimited = 2 });
numeric_enum!(CollectibleResaleRestriction { Invalid = 0, None = 1, Disabled = 2 });
numeric_enum!(CollectibleSaleStatus { OnSale = 0, OffSale = 1, Free = 2 });
numeric_enum!(CollectibleSaleLocationType { Invalid = 0, ShopAndAllExperiences = 1, ExperiencesAndDevApiOnly = 2, ShopOnly = 3, ShopAndExperiencesById = 4 });

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectibleSaleLocationInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sale_location_type: Option<CollectibleSaleLocationType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub places: Option<Vec<u64>>,
}
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublishCollectibleInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_rental_opt_in: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idempotency_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_type: Option<CollectibleTargetType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publishing_type: Option<CollectiblePublishingType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agreed_publishing_fee: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub creator_user_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub creator_group_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publisher_user_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantity: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantity_limit_per_user: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resale_restriction: Option<CollectibleResaleRestriction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_in_robux: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_offset: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opt_out_from_regional_pricing: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_free: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sale_location_configuration: Option<CollectibleSaleLocationInput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub on_sale_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub off_sale_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agreement_id: Option<String>,
}
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectibleTargetInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_type: Option<CollectibleTargetType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_id: Option<String>,
}
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BulkUpdateTargetsInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset_type: Option<Vec<u32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<Vec<CollectibleTargetInput>>,
}
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BulkUpdatePayloadInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rental_opt_in: Option<bool>,
}
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BulkUpdateCollectiblesInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_targets: Option<BulkUpdateTargetsInput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_payload: Option<BulkUpdatePayloadInput>,
}
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCollectibleInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sale_location_configuration: Option<CollectibleSaleLocationInput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_rental_opt_in: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sale_status: Option<CollectibleSaleStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantity_limit_per_user: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resale_restriction: Option<CollectibleResaleRestriction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_in_robux: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_offset: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opt_out_from_regional_pricing: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_free: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub on_sale_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub off_sale_time: Option<String>,
}
#[derive(Debug, Clone)]
pub struct CollectibleClient {
    transport: AsyncClient,
}
impl CollectibleClient {
    pub fn new(transport: AsyncClient) -> Self {
        Self { transport }
    }
    pub async fn publish(&self, input: PublishCollectibleInput) -> Result<Value, Error> {
        let request =
            Request::new(Operation::post(Server::Apis, "/v1/collectibles")).json(&input)?;
        self.transport.execute(request).await
    }
    pub async fn bulk_update(&self, input: BulkUpdateCollectiblesInput) -> Result<Value, Error> {
        let request = Request::new(Operation::patch(
            Server::Apis,
            "/v1/collectibles/bulk-update",
        ))
        .json(&input)?;
        self.transport.execute(request).await
    }
    pub async fn update(&self, id: &str, input: UpdateCollectibleInput) -> Result<Value, Error> {
        let request = Request::new(Operation::patch(
            Server::Apis,
            "/v1/collectibles/{collectibleItemId}",
        ))
        .path_param("collectibleItemId", id)
        .json(&input)?;
        self.transport.execute(request).await
    }
}
