//! Typed virtual event creation and partial updates.

use super::client::{AsyncClient, Error, Operation, Request, Server};
use serde::{Deserialize, Serialize};
use serde_json::Value;
#[derive(Debug, Clone, Copy, clap::ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EventVisibilityInput {
    Private,
    Public,
}
impl std::fmt::Display for EventVisibilityInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Private => "private",
                Self::Public => "public",
            }
        )
    }
}
#[derive(Debug, Clone, Copy, clap::ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EventFeaturingStatus {
    Enabled,
    Disabled,
}
impl std::fmt::Display for EventFeaturingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Enabled => "enabled",
                Self::Disabled => "disabled",
            }
        )
    }
}
#[derive(Debug, Clone, Copy, clap::ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EventCategoryInput {
    ContentUpdate,
    LocationUpdate,
    SystemUpdate,
    Activity,
    NewContent,
    ItemDrop,
    NewSeason,
    NewLocation,
    NewMap,
    MoreLevels,
    NewFeature,
    EarlyAccess,
    Expansion,
    Challenge,
    Quest,
    Festival,
}
impl std::fmt::Display for EventCategoryInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = serde_json::to_value(self).map_err(|_| std::fmt::Error)?;
        f.write_str(value.as_str().ok_or(std::fmt::Error)?)
    }
}
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EventRankedCategoryInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<EventCategoryInput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rank: Option<i32>,
}
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EventMediaInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rank: Option<i32>,
}
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateVirtualEventInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtitle: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visibility: Option<EventVisibilityInput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub place_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub categories: Option<Vec<EventRankedCategoryInput>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnails: Option<Vec<EventMediaInput>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub featuring_status: Option<EventFeaturingStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tagline: Option<String>,
}
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateVirtualEventInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtitle: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visibility: Option<EventVisibilityInput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub place_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub categories: Option<Vec<EventRankedCategoryInput>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnails: Option<Vec<EventMediaInput>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub featuring_status: Option<EventFeaturingStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tagline: Option<String>,
}
#[derive(Debug, Clone)]
pub struct VirtualEventClient {
    transport: AsyncClient,
}
impl VirtualEventClient {
    pub fn new(transport: AsyncClient) -> Self {
        Self { transport }
    }
    pub async fn create(
        &self,
        universe_id: u64,
        input: CreateVirtualEventInput,
    ) -> Result<Value, Error> {
        let request = Request::new(Operation::post(
            Server::Apis,
            "/virtual-events/v3/universes/{universeId}/game-events",
        ))
        .path_param("universeId", universe_id)
        .json(&input)?;
        self.transport.execute(request).await
    }
    pub async fn update(
        &self,
        event_id: &str,
        input: UpdateVirtualEventInput,
    ) -> Result<Value, Error> {
        let request = Request::new(Operation::patch(
            Server::Apis,
            "/virtual-events/v3/game-events/{eventId}",
        ))
        .path_param("eventId", event_id)
        .json(&input)?;
        self.transport.execute(request).await
    }
}
