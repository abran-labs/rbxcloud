//! Documented single-file Assets create and operation-status requests.
use serde::{Deserialize, Serialize};

use super::client::{AsyncClient, Attachment, Error, Operation, Request, Server};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, clap::ValueEnum)]
pub enum AssetKind {
    Image,
    Decal,
    Audio,
    Model,
}

impl AssetKind {
    pub fn content_type(self, extension: &str) -> Option<&'static str> {
        match (self, extension.to_ascii_lowercase().as_str()) {
            (Self::Image | Self::Decal, "png") => Some("image/png"),
            (Self::Image | Self::Decal, "jpg" | "jpeg") => Some("image/jpeg"),
            (Self::Image | Self::Decal, "bmp") => Some("image/bmp"),
            (Self::Image | Self::Decal, "tga") => Some("image/tga"),
            (Self::Audio, "mp3") => Some("audio/mpeg"),
            (Self::Audio, "ogg") => Some("audio/ogg"),
            (Self::Audio, "wav") => Some("audio/wav"),
            (Self::Audio, "flac") => Some("audio/flac"),
            (Self::Model, "fbx") => Some("model/fbx"),
            (Self::Model, "gltf") => Some("model/gltf+json"),
            (Self::Model, "glb") => Some("model/gltf-binary"),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, clap::ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum CreatorKind {
    User,
    Group,
}

#[derive(Serialize)]
#[serde(untagged)]
enum Creator {
    User {
        #[serde(rename = "userId")]
        user_id: String,
    },
    Group {
        #[serde(rename = "groupId")]
        group_id: String,
    },
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CreationContext {
    creator: Creator,
    expected_price: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Metadata<'a> {
    asset_type: AssetKind,
    display_name: &'a str,
    description: &'a str,
    creation_context: CreationContext,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetOperation {
    pub path: Option<String>,
    pub operation_id: Option<String>,
    #[serde(default)]
    pub done: bool,
    pub response: Option<CreatedAsset>,
    pub error: Option<serde::de::IgnoredAny>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatedAsset {
    pub asset_id: Option<String>,
    pub path: Option<String>,
}

#[derive(Clone)]
pub struct AssetCreateClient(AsyncClient);

pub struct CreateAsset<'a> {
    pub kind: AssetKind,
    pub creator: CreatorKind,
    pub creator_id: u64,
    pub display_name: &'a str,
    pub description: &'a str,
    pub file_name: &'a str,
    pub data: Vec<u8>,
}

impl AssetCreateClient {
    pub fn new(transport: AsyncClient) -> Self {
        Self(transport)
    }

    pub async fn create(&self, input: CreateAsset<'_>) -> Result<AssetOperation, Error> {
        let op = Operation::post(Server::Apis, "/assets/v1/assets");
        let extension = std::path::Path::new(input.file_name)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        let mime = input
            .kind
            .content_type(extension)
            .ok_or_else(|| Error::InvalidRequest {
                operation: op.label(),
                reason: "asset type does not support this extension".into(),
            })?;
        let creator = match input.creator {
            CreatorKind::User => Creator::User {
                user_id: input.creator_id.to_string(),
            },
            CreatorKind::Group => Creator::Group {
                group_id: input.creator_id.to_string(),
            },
        };
        let metadata = Metadata {
            asset_type: input.kind,
            display_name: input.display_name,
            description: input.description,
            creation_context: CreationContext {
                creator,
                expected_price: 0,
            },
        };
        let attachment = Attachment::from_bytes(input.file_name, mime, input.data)?;
        self.execute(
            Request::new(op)
                .multipart_json("request", &metadata)?
                .multipart_file("fileContent", attachment),
        )
        .await
    }

    pub async fn operation(&self, operation_id: &str) -> Result<AssetOperation, Error> {
        self.execute(
            Request::new(Operation::get(
                Server::Apis,
                "/assets/v1/operations/{operationId}",
            ))
            .path_param("operationId", operation_id),
        )
        .await
    }

    async fn execute(&self, request: Request) -> Result<AssetOperation, Error> {
        let operation = request.operation.label();
        let result = self.0.execute(request).await?;
        serde_json::from_value(result).map_err(|error| Error::InvalidResponse {
            operation,
            status: 200,
            reason: format!("unexpected asset operation ({:?})", error.classify()),
        })
    }
}
