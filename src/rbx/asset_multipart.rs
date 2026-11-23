//! Typed start operations for Roblox multipart asset uploads.
use super::{
    client::{AsyncClient, Error, Operation, Request, Server},
    generated::multipart_models::{AssetsUploadApiAsset, CreationContext},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadFileInput {
    pub content_type: Option<String>,
    pub filesize: Option<i64>,
    pub md5_checksum: Option<String>,
    pub chunk_plan: Option<Vec<i64>>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MultipartStartInput {
    pub asset: AssetsUploadApiAsset,
    pub file: UploadFileInput,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreSignedUploadUrl {
    pub http_verb: Option<String>,
    pub url: Option<String>,
    pub expiration_time_ms: Option<String>,
    pub chunk_num: Option<String>,
    pub content_start: Option<String>,
    pub content_length: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MultipartStartResponse {
    pub operation_path: Option<String>,
    pub upload_urls: Option<Vec<PreSignedUploadUrl>>,
}

#[derive(Debug, Clone)]
pub struct AssetMultipartClient {
    transport: AsyncClient,
}
impl AssetMultipartClient {
    pub fn new(transport: AsyncClient) -> Self {
        Self { transport }
    }
    pub async fn create(
        &self,
        input: MultipartStartInput,
    ) -> Result<MultipartStartResponse, Error> {
        self.start(
            Operation::post(Server::Apis, "/assets/v1/assets:multipartUpload"),
            None,
            input,
        )
        .await
    }
    pub async fn update(
        &self,
        asset_id: u64,
        input: MultipartStartInput,
    ) -> Result<MultipartStartResponse, Error> {
        self.start(
            Operation::patch(Server::Apis, "/assets/v1/assets/{assetId}:multipartUpload"),
            Some(asset_id),
            input,
        )
        .await
    }
    async fn start(
        &self,
        operation: Operation,
        asset_id: Option<u64>,
        input: MultipartStartInput,
    ) -> Result<MultipartStartResponse, Error> {
        let mut request = Request::new(operation);
        if let Some(id) = asset_id {
            request = request.path_param("assetId", id);
        }
        let value = self.transport.execute(request.json(&input)?).await?;
        serde_json::from_value(value).map_err(|error| Error::InvalidResponse {
            operation: "multipart upload start".into(),
            status: 200,
            reason: error.to_string(),
        })
    }
}

pub fn asset_metadata(
    display_name: String,
    description: String,
    asset_type: String,
    creator: super::generated::multipart_models::AssetsUploadApiCreator,
    expected_price: Option<i64>,
) -> AssetsUploadApiAsset {
    AssetsUploadApiAsset {
        display_name: Some(display_name),
        description: Some(description),
        asset_type: Some(asset_type),
        creation_context: Some(CreationContext {
            creator: Some(creator),
            expected_price,
            ..Default::default()
        }),
        ..Default::default()
    }
}
