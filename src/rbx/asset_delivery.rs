//! Authenticated binary Roblox asset downloads.
use super::client::{AsyncClient, BinaryResponse, Error, Operation, Request, Server};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default)]
pub struct AssetDownloadInput {
    pub asset_id: u64,
    pub version_number: Option<u32>,
    pub accept_encoding: Option<String>,
    pub place_id: Option<u64>,
    pub asset_type: Option<String>,
    pub asset_format: Option<String>,
    pub roblox_asset_format: Option<String>,
    pub usage_context: Option<String>,
}

#[derive(Debug)]
pub struct DownloadedAsset {
    pub path: PathBuf,
    pub content_type: Option<String>,
    pub byte_length: usize,
}

#[derive(Debug, Clone)]
pub struct AssetDeliveryClient {
    transport: AsyncClient,
}
impl AssetDeliveryClient {
    pub fn new(transport: AsyncClient) -> Self {
        Self { transport }
    }
    pub async fn download(
        &self,
        input: AssetDownloadInput,
        output: &Path,
    ) -> Result<DownloadedAsset, Error> {
        if output.exists() || output.symlink_metadata().is_ok() {
            return Err(Error::InvalidRequest {
                operation: "asset download".into(),
                reason: "output already exists; choose a new path".into(),
            });
        }
        let (operation, path) = match input.version_number {
            Some(version) => (
                Operation::get(
                    Server::Apis,
                    "/asset-delivery-api/v1/assetId/{assetId}/version/{versionNumber}",
                ),
                vec![
                    ("assetId", input.asset_id.to_string()),
                    ("versionNumber", version.to_string()),
                ],
            ),
            None => (
                Operation::get(Server::Apis, "/asset-delivery-api/v1/assetId/{assetId}"),
                vec![("assetId", input.asset_id.to_string())],
            ),
        };
        let mut request = Request::new(operation);
        for (name, value) in path {
            request = request.path_param(name, value);
        }
        request = request
            .header_opt("Accept-Encoding", input.accept_encoding)
            .header_opt("Roblox-Place-Id", input.place_id)
            .header_opt("AssetType", input.asset_type)
            .header_opt("AssetFormat", input.asset_format)
            .header_opt("Roblox-AssetFormat", input.roblox_asset_format)
            .header_opt("Roblox-Usage-Context", input.usage_context);
        let BinaryResponse {
            bytes,
            content_type,
        } = self.transport.execute_binary(request).await?;
        let parent = output.parent().unwrap_or_else(|| Path::new("."));
        if !parent.is_dir() {
            return Err(Error::InvalidRequest {
                operation: "asset download".into(),
                reason: "output parent directory does not exist".into(),
            });
        }
        let file_name =
            output
                .file_name()
                .and_then(|v| v.to_str())
                .ok_or_else(|| Error::InvalidRequest {
                    operation: "asset download".into(),
                    reason: "output must be a regular file path".into(),
                })?;
        let temp = parent.join(format!(".{file_name}.rbxcloud-{}.tmp", std::process::id()));
        std::fs::write(&temp, &bytes).map_err(|error| Error::InvalidRequest {
            operation: "asset download".into(),
            reason: format!("could not stage output: {}", error.kind()),
        })?;
        if let Err(error) = std::fs::rename(&temp, output) {
            let _ = std::fs::remove_file(&temp);
            return Err(Error::InvalidRequest {
                operation: "asset download".into(),
                reason: format!("could not publish output: {}", error.kind()),
            });
        }
        Ok(DownloadedAsset {
            path: output.to_path_buf(),
            content_type,
            byte_length: bytes.len(),
        })
    }
}
