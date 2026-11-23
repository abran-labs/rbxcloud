//! Typed batch asset permission grants.

use super::client::{AsyncClient, Error, Operation, Request, Server};
use serde::{Deserialize, Serialize};
use serde_json::Value;

macro_rules! string_enum {
    ($name:ident { $($variant:ident),+ $(,)? }) => {
        #[derive(Debug, Clone, Copy, clap::ValueEnum, Serialize, Deserialize)]
        pub enum $name { $($variant),+ }
        impl std::fmt::Display for $name { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{:?}", self) } }
    };
}
string_enum!(AssetSubjectType {
    Invalid,
    User,
    Group,
    GroupRoleset,
    All,
    Universe,
    CommunicationPlatformMessage,
    IpPlatform
});
string_enum!(AssetGrantAction {
    Invalid,
    Edit,
    Use,
    Download,
    CopyFromRcc,
    UpdateFromRcc
});

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetGrantInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grant_to_dependencies: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_version_number: Option<u32>,
}
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchGrantPermissionsInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject_type: Option<AssetSubjectType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<AssetGrantAction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requests: Option<Vec<AssetGrantInput>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset_ids: Option<Vec<u64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_deep_access_check: Option<bool>,
}
#[derive(Debug, Clone)]
pub struct AssetPermissionClient {
    transport: AsyncClient,
}
impl AssetPermissionClient {
    pub fn new(transport: AsyncClient) -> Self {
        Self { transport }
    }
    pub async fn batch_grant(&self, input: BatchGrantPermissionsInput) -> Result<Value, Error> {
        let request = Request::new(Operation::patch(
            Server::Apis,
            "/asset-permissions-api/v1/assets/permissions",
        ))
        .json(&input)?;
        self.transport.execute(request).await
    }
}
