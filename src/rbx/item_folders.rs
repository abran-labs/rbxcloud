//! Typed item-folder additions and removals.

use super::client::{AsyncClient, Error, Operation, Request, Server};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Copy, clap::ValueEnum, Serialize, Deserialize)]
pub enum FolderItemType {
    Unknown,
    Asset,
    Bundle,
    IECToken,
    Folder,
}
impl std::fmt::Display for FolderItemType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FolderItemInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item_type: Option<FolderItemType>,
}
#[derive(Serialize)]
struct FolderItemsBody {
    items: Vec<FolderItemInput>,
}

#[derive(Debug, Clone)]
pub struct ItemFolderClient {
    transport: AsyncClient,
}
impl ItemFolderClient {
    pub fn new(transport: AsyncClient) -> Self {
        Self { transport }
    }
    pub async fn add_items(
        &self,
        folder_id: String,
        items: Vec<FolderItemInput>,
    ) -> Result<Value, Error> {
        self.call(
            Operation::post(Server::Apis, "/v1/folders/{folderId}/items"),
            folder_id,
            items,
        )
        .await
    }
    pub async fn delete_items(
        &self,
        folder_id: String,
        items: Vec<FolderItemInput>,
    ) -> Result<Value, Error> {
        self.call(
            Operation::delete(Server::Apis, "/v1/folders/{folderId}/items"),
            folder_id,
            items,
        )
        .await
    }
    async fn call(
        &self,
        operation: Operation,
        folder_id: String,
        items: Vec<FolderItemInput>,
    ) -> Result<Value, Error> {
        let request = Request::new(operation)
            .path_param("folderId", folder_id)
            .json(&FolderItemsBody { items })?;
        self.transport.execute(request).await
    }
}
