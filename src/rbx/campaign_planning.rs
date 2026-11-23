//! Typed advertising campaign planning requests.
use super::client::{AsyncClient, Error, Operation, Request, Server};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, clap::ValueEnum, Serialize, Deserialize)]
pub enum CampaignObjective {
    #[serde(rename = "PLAYS")]
    Plays,
    #[serde(rename = "ENGAGEMENT")]
    Engagement,
    #[serde(rename = "ROBUX_SPEND")]
    RobuxSpend,
}
impl std::fmt::Display for CampaignObjective {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Plays => "PLAYS",
            Self::Engagement => "ENGAGEMENT",
            Self::RobuxSpend => "ROBUX_SPEND",
        })
    }
}

#[derive(Debug, Clone, Copy, clap::ValueEnum, Serialize, Deserialize)]
pub enum AudienceType {
    #[serde(rename = "ALL_PLAYERS")]
    AllPlayers,
    #[serde(rename = "NEW_USERS")]
    NewUsers,
    #[serde(rename = "RETENTION")]
    Retention,
    #[serde(rename = "REACTIVATION")]
    Reactivation,
}
impl std::fmt::Display for AudienceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::AllPlayers => "ALL_PLAYERS",
            Self::NewUsers => "NEW_USERS",
            Self::Retention => "RETENTION",
            Self::Reactivation => "REACTIVATION",
        })
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanningTargeting {
    pub audience_type: Option<AudienceType>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanningContext {
    pub objective: CampaignObjective,
    pub target_universe_id: String,
    pub targeting: Option<PlanningTargeting>,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct RecommendationRequest {
    campaign_context: PlanningContext,
}

#[derive(Debug, Clone)]
pub struct CampaignPlanningClient {
    transport: AsyncClient,
}
impl CampaignPlanningClient {
    pub fn new(transport: AsyncClient) -> Self {
        Self { transport }
    }
    pub async fn recommend(&self, context: PlanningContext) -> Result<serde_json::Value, Error> {
        let request = Request::new(Operation::post(
            Server::Apis,
            "/ads-management/v1/campaign-recommendations",
        ))
        .json(&RecommendationRequest {
            campaign_context: context,
        })?;
        self.transport.execute(request).await
    }
}
