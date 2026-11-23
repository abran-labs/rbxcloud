//! Typed custom analytics dashboard validation, creation, metadata, and publishing.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::client::{AsyncClient, Error, Operation, Request, Server};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CustomDashboardDocumentInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema_version: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DashboardMetadataPatchInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_pinned: Option<bool>,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateDashboardInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document: Option<CustomDashboardDocumentInput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDashboardMetadataInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_head_etag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub patch: Option<DashboardMetadataPatchInput>,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublishDashboardInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_head_etag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document: Option<CustomDashboardDocumentInput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata_patch: Option<DashboardMetadataPatchInput>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ValidateDocumentBody {
    document: CustomDashboardDocumentInput,
}

#[derive(Debug, Clone)]
pub struct AnalyticsDashboardClient {
    transport: AsyncClient,
}

impl AnalyticsDashboardClient {
    pub fn new(transport: AsyncClient) -> Self {
        Self { transport }
    }

    pub async fn validate_document(
        &self,
        universe_id: u64,
        document: CustomDashboardDocumentInput,
    ) -> Result<Value, Error> {
        let request = Request::new(Operation::post(Server::Apis, "/analytics-custom-dashboards/v1/universes/{universeId}/custom-dashboard-documents/validate"))
            .path_param("universeId", universe_id).json(&ValidateDocumentBody { document })?;
        self.transport.execute(request).await
    }

    pub async fn create_dashboard(
        &self,
        universe_id: u64,
        input: CreateDashboardInput,
    ) -> Result<Value, Error> {
        let request = Request::new(Operation::post(
            Server::Apis,
            "/analytics-custom-dashboards/v1/universes/{universeId}/custom-dashboards",
        ))
        .path_param("universeId", universe_id)
        .json(&input)?;
        self.transport.execute(request).await
    }

    pub async fn update_metadata(
        &self,
        universe_id: u64,
        dashboard_id: String,
        input: UpdateDashboardMetadataInput,
    ) -> Result<Value, Error> {
        let request = Request::new(Operation::patch(Server::Apis, "/analytics-custom-dashboards/v1/universes/{universeId}/custom-dashboards/{dashboardId}/metadata"))
            .path_param("universeId", universe_id).path_param("dashboardId", dashboard_id).json(&input)?;
        self.transport.execute(request).await
    }

    pub async fn publish_dashboard(
        &self,
        universe_id: u64,
        dashboard_id: String,
        input: PublishDashboardInput,
    ) -> Result<Value, Error> {
        let request = Request::new(Operation::post(Server::Apis, "/analytics-custom-dashboards/v1/universes/{universeId}/custom-dashboards/{dashboardId}/publish"))
            .path_param("universeId", universe_id).path_param("dashboardId", dashboard_id).json(&input)?;
        self.transport.execute(request).await
    }
}
