//! Explicit typed clients for newer Open Cloud resources.
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::client::{AsyncClient, Error, Operation, Request, Server};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataStoreEntry {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub etag: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub users: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attributes: Option<Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListDataStoresResponse {
    #[serde(default)]
    pub data_stores: Vec<DataStore>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_page_token: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataStore {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IncrementDataStoreEntryRequest {
    pub amount: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotDataStoresRequest {}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotDataStoresResponse {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub new_snapshot_taken: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_snapshot_time: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ListDataStoresParams {
    pub universe_id: String,
    pub max_page_size: Option<u32>,
    pub page_token: Option<String>,
    pub filter: Option<String>,
    pub show_deleted: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct DataStoreEntryPath {
    pub universe_id: String,
    pub data_store_id: String,
    pub entry_id: String,
}

#[derive(Debug, Clone)]
pub struct CloudClient {
    transport: AsyncClient,
}

impl CloudClient {
    pub fn new(transport: AsyncClient) -> Self {
        Self { transport }
    }

    pub async fn list_data_stores(
        &self,
        params: ListDataStoresParams,
    ) -> Result<ListDataStoresResponse, Error> {
        let request = Request::new(Operation::get(
            Server::Apis,
            "/cloud/v2/universes/{universe_id}/data-stores",
        ))
        .path_param("universe_id", params.universe_id)
        .query_opt("maxPageSize", params.max_page_size)
        .query_opt("pageToken", params.page_token)
        .query_opt("filter", params.filter)
        .query_opt("showDeleted", params.show_deleted);
        self.decode(self.transport.execute(request).await?)
    }

    pub async fn get_data_store_entry(
        &self,
        path: DataStoreEntryPath,
    ) -> Result<DataStoreEntry, Error> {
        let request = Request::new(Operation::get(
            Server::Apis,
            "/cloud/v2/universes/{universe_id}/data-stores/{data_store_id}/entries/{entry_id}",
        ))
        .path_param("universe_id", path.universe_id)
        .path_param("data_store_id", path.data_store_id)
        .path_param("entry_id", path.entry_id);
        self.decode(self.transport.execute(request).await?)
    }

    pub async fn create_data_store_entry(
        &self,
        universe_id: String,
        data_store_id: String,
        id: String,
        entry: DataStoreEntry,
    ) -> Result<DataStoreEntry, Error> {
        let request = Request::new(Operation::post(
            Server::Apis,
            "/cloud/v2/universes/{universe_id}/data-stores/{data_store_id}/entries",
        ))
        .path_param("universe_id", universe_id)
        .path_param("data_store_id", data_store_id)
        .query("id", id)
        .json(&entry)?;
        self.decode(self.transport.execute(request).await?)
    }

    pub async fn delete_data_store_entry(&self, path: DataStoreEntryPath) -> Result<(), Error> {
        let request = Request::new(Operation::delete(
            Server::Apis,
            "/cloud/v2/universes/{universe_id}/data-stores/{data_store_id}/entries/{entry_id}",
        ))
        .path_param("universe_id", path.universe_id)
        .path_param("data_store_id", path.data_store_id)
        .path_param("entry_id", path.entry_id);
        self.transport.execute(request).await?;
        Ok(())
    }

    pub async fn increment_data_store_entry(
        &self,
        path: DataStoreEntryPath,
        amount: i64,
    ) -> Result<DataStoreEntry, Error> {
        let body = IncrementDataStoreEntryRequest { amount };
        let request = Request::new(Operation::post(Server::Apis, "/cloud/v2/universes/{universe_id}/data-stores/{data_store_id}/entries/{entry_id}:increment"))
            .path_param("universe_id", path.universe_id)
            .path_param("data_store_id", path.data_store_id)
            .path_param("entry_id", path.entry_id)
            .json(&body)?;
        self.decode(self.transport.execute(request).await?)
    }

    pub async fn snapshot_data_stores(
        &self,
        universe_id: String,
        body: SnapshotDataStoresRequest,
    ) -> Result<SnapshotDataStoresResponse, Error> {
        let request = Request::new(Operation::post(
            Server::Apis,
            "/cloud/v2/universes/{universe_id}/data-stores:snapshot",
        ))
        .path_param("universe_id", universe_id)
        .json(&body)?;
        self.decode(self.transport.execute(request).await?)
    }

    fn decode<T: for<'de> Deserialize<'de>>(&self, value: Value) -> Result<T, Error> {
        serde_json::from_value(value).map_err(|error| Error::InvalidResponse {
            operation: "typed Open Cloud response".into(),
            status: 200,
            reason: error.to_string(),
        })
    }
}
