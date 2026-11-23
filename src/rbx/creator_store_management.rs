//! Typed Creator Store product pricing, saved-item deletion, and asset search.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::client::{AsyncClient, Error, Operation, Request, Server};

macro_rules! string_enum {
    ($name:ident { $($variant:ident),+ $(,)? }) => {
        #[derive(Debug, Clone, Copy, clap::ValueEnum, Serialize, Deserialize)]
        pub enum $name { $($variant),+ }
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{:?}", self) }
        }
    };
}

string_enum!(SearchCategoryType {
    Audio,
    Place,
    Model,
    Decal,
    Plugin,
    MeshPart,
    Video,
    FontFamily
});
string_enum!(CreatorStoreSortDirection {
    None,
    Ascending,
    Descending
});
string_enum!(CreatorStoreSortCategory {
    Relevance,
    Trending,
    Top,
    AudioDuration,
    CreateTime,
    UpdatedTime,
    Ratings
});
string_enum!(CreatorStoreSearchView { IDs, Core, Full });
string_enum!(SearchAudioType {
    Music,
    SoundEffect,
    Unknown
});
string_enum!(MusicChartType {
    None,
    Current,
    Week,
    Month,
    Year
});
string_enum!(ModelSubType {
    Ad,
    MaterialPack,
    Package
});
string_enum!(ModelInstanceType {
    Script,
    MeshPart,
    Decal,
    Animation,
    Audio,
    Tool
});

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DecimalInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub significand: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exponent: Option<i32>,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MoneyInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantity: Option<DecimalInput>,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCreatorStoreProductInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_price: Option<MoneyInput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub purchase_price: Option<MoneyInput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_asset_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plugin_asset_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio_asset_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decal_asset_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mesh_part_asset_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub video_asset_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_family_asset_id: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCreatorStoreProductInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_price: Option<MoneyInput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub purchase_price: Option<MoneyInput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteSaveInput {
    pub target_type: SearchCategoryType,
    pub target_id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collection_name: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioSearchFiltersInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio_types: Option<Vec<SearchAudioType>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_duration_seconds: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_duration_seconds: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artist: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub album: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub music_chart_type: Option<MusicChartType>,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelSearchFiltersInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub included_sub_types: Option<Vec<ModelSubType>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub excluded_sub_types: Option<Vec<ModelSubType>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub included_instance_types: Option<Vec<ModelInstanceType>>,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchCreatorStoreAssetsInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_category_type: Option<SearchCategoryType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_number: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_page_size: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_direction: Option<CreatorStoreSortDirection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_category: Option<CreatorStoreSortCategory>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_only_verified_creators: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_price_cents: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_price_cents: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub facets: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub swimlane: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_view: Option<CreatorStoreSearchView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio_search_filters: Option<AudioSearchFiltersInput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_search_filters: Option<ModelSearchFiltersInput>,
}

#[derive(Serialize)]
struct BulkDeleteBody {
    targets: Vec<DeleteSaveInput>,
}

#[derive(Debug, Clone)]
pub struct CreatorStoreProductClient {
    transport: AsyncClient,
}
impl CreatorStoreProductClient {
    pub fn new(transport: AsyncClient) -> Self {
        Self { transport }
    }
    pub async fn create_product(
        &self,
        input: CreateCreatorStoreProductInput,
    ) -> Result<Value, Error> {
        let request = Request::new(Operation::post(
            Server::Apis,
            "/cloud/v2/creator-store-products",
        ))
        .json(&input)?;
        self.transport.execute(request).await
    }
    pub async fn update_product(
        &self,
        id: String,
        input: UpdateCreatorStoreProductInput,
    ) -> Result<Value, Error> {
        let request = Request::new(Operation::patch(
            Server::Apis,
            "/cloud/v2/creator-store-products/{creator_store_product_id}",
        ))
        .path_param("creator_store_product_id", id)
        .json(&input)?;
        self.transport.execute(request).await
    }
}

#[derive(Debug, Clone)]
pub struct ToolboxClient {
    transport: AsyncClient,
}
impl ToolboxClient {
    pub fn new(transport: AsyncClient) -> Self {
        Self { transport }
    }
    pub async fn bulk_delete_saves(&self, targets: Vec<DeleteSaveInput>) -> Result<Value, Error> {
        let request = Request::new(Operation::post(
            Server::Apis,
            "/toolbox-service/v1/saves:bulkDelete",
        ))
        .json(&BulkDeleteBody { targets })?;
        self.transport.execute(request).await
    }
    pub async fn search_assets(
        &self,
        input: SearchCreatorStoreAssetsInput,
    ) -> Result<Value, Error> {
        let request = Request::new(Operation::post(
            Server::Apis,
            "/toolbox-service/v2/assets:search",
        ))
        .json(&input)?;
        self.transport.execute(request).await
    }
}
