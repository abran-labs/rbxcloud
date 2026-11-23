//! Typed advertising campaign creation, validation, updating, and estimates.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{
    campaign_planning::{AudienceType, CampaignObjective},
    client::{AsyncClient, Error, Operation, Request, Server},
};

macro_rules! string_enum {
    ($name:ident { $($variant:ident => $wire:literal),+ $(,)? }) => {
        #[derive(Debug, Clone, Copy, clap::ValueEnum, Serialize, Deserialize)]
        pub enum $name { $(#[serde(rename = $wire)] $variant),+ }
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(match self { $(Self::$variant => $wire),+ })
            }
        }
    };
}

string_enum!(AgeGroup { Age13To17 => "AGE_13_17", Age18To24 => "AGE_18_24", Age25Plus => "AGE_25_PLUS" });
string_enum!(Device { Phone => "PHONE", Tablet => "TABLET", Desktop => "DESKTOP", Console => "CONSOLE" });
string_enum!(Region { All => "ALL", WesternEurope => "WESTERN_EUROPE", EasternEurope => "EASTERN_EUROPE", EastAsia => "EAST_ASIA", SoutheastAsia => "SOUTHEAST_ASIA", MiddleEast => "MIDDLE_EAST", LatinAmerica => "LATIN_AMERICA", Africa => "AFRICA", Anz => "ANZ", Row => "ROW", Uac => "UAC" });
string_enum!(BudgetType { Daily => "DAILY", Lifetime => "LIFETIME" });
string_enum!(PaymentType { CreditCard => "CREDIT_CARD", AdsCredit => "ADS_CREDIT", Invoice => "INVOICE" });
string_enum!(CampaignStatus { Active => "ACTIVE", Paused => "PAUSED", Cancelled => "CANCELLED" });
string_enum!(VideoSource { ImageOnly => "IMAGE_ONLY", PreviewVideo => "PREVIEW_VIDEO" });
string_enum!(PerformanceObjective { Plays => "PLAYS" });

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TargetingInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub age_groups: Option<Vec<AgeGroup>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audience_type: Option<AudienceType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub countries: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub devices: Option<Vec<Device>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regions: Option<Vec<Region>>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AudienceEstimateInput {
    pub objective: CampaignObjective,
    pub target_universe_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub targeting: Option<TargetingInput>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BudgetInput {
    pub amount_micros: String,
    #[serde(rename = "type")]
    pub budget_type: BudgetType,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduleInput {
    pub duration_in_days: u32,
    pub start_time: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct VideoInput {
    pub source: VideoSource,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CampaignInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_reload_ad_credit_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub billing_account_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub budget: Option<BudgetInput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub creative_asset_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub objective: Option<CampaignObjective>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_type: Option<PaymentType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schedule: Option<ScheduleInput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_universe_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub targeting: Option<TargetingInput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub video: Option<VideoInput>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateBudgetInput {
    pub amount_micros: String,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateScheduleInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_in_days: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_time: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CampaignUpdateInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_reload_ad_credit_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub budget: Option<UpdateBudgetInput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub creative_asset_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schedule: Option<UpdateScheduleInput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<CampaignStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub video: Option<VideoInput>,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PerformanceForecastInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub budget: Option<PerformanceBudgetInput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_outcome: Option<PerformanceTargetOutcome>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PerformanceBudgetInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount_micro_usd: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PerformanceTargetOutcome {
    pub outcome_type: PerformanceObjective,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PerformanceScope {
    pub budget_type: BudgetType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_in_days: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct CampaignClient {
    transport: AsyncClient,
}

impl CampaignClient {
    pub fn new(transport: AsyncClient) -> Self {
        Self { transport }
    }

    pub async fn estimate_audience(&self, context: AudienceEstimateInput) -> Result<Value, Error> {
        let body = serde_json::json!({ "campaignContext": context });
        self.call(
            Operation::post(Server::Apis, "/ads-management/v1/audience-estimates"),
            None,
            &body,
        )
        .await
    }

    pub async fn create(&self, input: CampaignInput) -> Result<Value, Error> {
        self.call(
            Operation::post(Server::Apis, "/ads-management/v1/campaigns"),
            None,
            &input,
        )
        .await
    }

    pub async fn validate(&self, input: CampaignInput) -> Result<Value, Error> {
        self.call(
            Operation::post(Server::Apis, "/ads-management/v1/campaigns:validate"),
            None,
            &input,
        )
        .await
    }

    pub async fn update(&self, id: String, input: CampaignUpdateInput) -> Result<Value, Error> {
        self.call(
            Operation::patch(Server::Apis, "/ads-management/v1/campaigns/{id}"),
            Some(id),
            &input,
        )
        .await
    }

    pub async fn estimate_performance(
        &self,
        input: PerformanceEstimateInput,
    ) -> Result<Value, Error> {
        let context = serde_json::json!({
            "objective": input.objective,
            "targetUniverseId": input.target_universe_id,
        });
        let body = serde_json::json!({
            "campaignContext": context,
            "forecastInput": input.forecast_input,
            "forecastScope": input.forecast_scope,
        });
        self.call(
            Operation::post(Server::Apis, "/ads-management/v1/performance-estimates"),
            None,
            &body,
        )
        .await
    }

    async fn call(
        &self,
        operation: Operation,
        id: Option<String>,
        body: &impl Serialize,
    ) -> Result<Value, Error> {
        let mut request = Request::new(operation);
        if let Some(id) = id {
            request = request.path_param("id", id);
        }
        self.transport.execute(request.json(body)?).await
    }
}

#[derive(Debug, Clone)]
pub struct PerformanceEstimateInput {
    pub objective: PerformanceObjective,
    pub target_universe_id: String,
    pub forecast_input: PerformanceForecastInput,
    pub forecast_scope: PerformanceScope,
}
