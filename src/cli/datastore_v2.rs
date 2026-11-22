use clap::{Args, Subcommand};
use rbxcloud::rbx::{
    client::{AsyncClient, ClientPolicy},
    cloud::{
        CloudClient, DataStoreEntry, DataStoreEntryPath, ListDataStoresParams,
        SnapshotDataStoresRequest,
    },
    credentials::Credentials,
};
use serde::Serialize;

#[derive(Debug, Args)]
pub struct DataStoreV2 {
    #[command(subcommand)]
    command: DataStoreV2Command,
}

#[derive(Debug, Subcommand)]
enum DataStoreV2Command {
    /// List v2 DataStores in a universe
    ListDataStores {
        universe_id: String,
        #[arg(long)]
        max_page_size: Option<u32>,
        #[arg(long)]
        page_token: Option<String>,
        #[arg(long)]
        filter: Option<String>,
        #[arg(long)]
        show_deleted: bool,
    },
    /// Get a v2 DataStore entry
    GetDataStoreEntry {
        universe_id: String,
        data_store_id: String,
        entry_id: String,
    },
    /// Create a v2 DataStore entry with an arbitrary Luau JSON value
    CreateDataStoreEntry {
        universe_id: String,
        data_store_id: String,
        #[arg(long)]
        id: String,
        #[arg(long)]
        value_json: String,
        #[arg(long)]
        user: Vec<String>,
    },
    /// Delete a v2 DataStore entry
    DeleteDataStoreEntry {
        universe_id: String,
        data_store_id: String,
        entry_id: String,
    },
    /// Increment a v2 DataStore entry
    IncrementDataStoreEntry {
        universe_id: String,
        data_store_id: String,
        entry_id: String,
        #[arg(long)]
        amount: i64,
    },
    /// Take a v2 DataStore snapshot
    SnapshotDataStores { universe_id: String },
}

fn path(universe_id: String, data_store_id: String, entry_id: String) -> DataStoreEntryPath {
    DataStoreEntryPath {
        universe_id,
        data_store_id,
        entry_id,
    }
}

fn json<T: Serialize>(value: T) -> anyhow::Result<Option<String>> {
    Ok(Some(serde_json::to_string_pretty(&value)?))
}

impl DataStoreV2 {
    pub async fn run(self) -> anyhow::Result<Option<String>> {
        let credentials = Credentials::from_process()?;
        let transport = AsyncClient::new(credentials, ClientPolicy::default()).await?;
        let client = CloudClient::new(transport);
        match self.command {
            DataStoreV2Command::ListDataStores {
                universe_id,
                max_page_size,
                page_token,
                filter,
                show_deleted,
            } => json(
                client
                    .list_data_stores(ListDataStoresParams {
                        universe_id,
                        max_page_size,
                        page_token,
                        filter,
                        show_deleted: Some(show_deleted),
                    })
                    .await?,
            ),
            DataStoreV2Command::GetDataStoreEntry {
                universe_id,
                data_store_id,
                entry_id,
            } => json(
                client
                    .get_data_store_entry(path(universe_id, data_store_id, entry_id))
                    .await?,
            ),
            DataStoreV2Command::CreateDataStoreEntry {
                universe_id,
                data_store_id,
                id,
                value_json,
                user,
            } => {
                let entry = DataStoreEntry {
                    value: Some(serde_json::from_str(&value_json)?),
                    users: (!user.is_empty()).then_some(user),
                    ..Default::default()
                };
                json(
                    client
                        .create_data_store_entry(universe_id, data_store_id, id, entry)
                        .await?,
                )
            }
            DataStoreV2Command::DeleteDataStoreEntry {
                universe_id,
                data_store_id,
                entry_id,
            } => {
                client
                    .delete_data_store_entry(path(universe_id, data_store_id, entry_id))
                    .await?;
                Ok(None)
            }
            DataStoreV2Command::IncrementDataStoreEntry {
                universe_id,
                data_store_id,
                entry_id,
                amount,
            } => json(
                client
                    .increment_data_store_entry(path(universe_id, data_store_id, entry_id), amount)
                    .await?,
            ),
            DataStoreV2Command::SnapshotDataStores { universe_id } => json(
                client
                    .snapshot_data_stores(universe_id, SnapshotDataStoresRequest {})
                    .await?,
            ),
        }
    }
}
