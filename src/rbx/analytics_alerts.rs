//! Typed analytics alert configuration creation and partial updates.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::client::{AsyncClient, Error, Operation, Request, Server};

macro_rules! string_enum {
    ($name:ident { $($variant:ident => $wire:literal),+ $(,)? }) => {
        #[derive(Debug, Clone, Copy, clap::ValueEnum, Serialize, Deserialize)]
        pub enum $name { $(#[serde(rename = $wire)] $variant),+ }
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.write_str(match self { $(Self::$variant => $wire),+ }) }
        }
    };
}

string_enum!(AlertSeverity { Sev0 => "SEV_0", Sev1 => "SEV_1", Sev2 => "SEV_2" });
string_enum!(AlertInterval { OneMinute => "OneMinute", HalfHour => "HalfHour", OneHour => "OneHour", OneDay => "OneDay" });
string_enum!(ConditionOperator { Gt => "Gt", Gte => "Gte", Lt => "Lt", Lte => "Lte" });
string_enum!(EvaluationMode { Absolute => "Absolute", PeriodOverPeriod => "PeriodOverPeriod" });
string_enum!(AlertConfigState { Enabled => "Enabled", Disabled => "Disabled", PausedByRoblox => "PausedByRoblox", Syncing => "Syncing", Error => "Error" });

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AlertBreakdownInput {
    pub dimensions: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AlertFilterInput {
    pub dimension: String,
    pub values: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AlertConditionInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operator: Option<ConditionOperator>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub threshold: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evaluation_mode: Option<EvaluationMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub period_offset_multiplier: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WebhookReceiverInput {
    pub webhook_configuration_id: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WebhookReceiverConfigInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receivers: Option<Vec<WebhookReceiverInput>>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAlertInput {
    pub name: String,
    pub metric: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub severity: AlertSeverity,
    pub interval: AlertInterval,
    pub consecutive_occurrences: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub breakdown: Option<Vec<AlertBreakdownInput>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<Vec<AlertFilterInput>>,
    pub condition: AlertConditionInput,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook_receiver_config: Option<WebhookReceiverConfigInput>,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAlertInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metric: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub severity: Option<AlertSeverity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interval: Option<AlertInterval>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub consecutive_occurrences: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub breakdown: Option<Vec<AlertBreakdownInput>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<Vec<AlertFilterInput>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition: Option<AlertConditionInput>,
    /// `None` leaves receivers unchanged; `Some(None)` disables notifications.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook_receiver_config: Option<Option<WebhookReceiverConfigInput>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_state: Option<AlertConfigState>,
}

#[derive(Debug, Clone)]
pub struct AnalyticsAlertClient {
    transport: AsyncClient,
}

impl AnalyticsAlertClient {
    pub fn new(transport: AsyncClient) -> Self {
        Self { transport }
    }

    pub async fn create_alert(
        &self,
        resource_type: &str,
        resource_id: &str,
        input: CreateAlertInput,
    ) -> Result<Value, Error> {
        let request = Request::new(Operation::post(
            Server::Apis,
            "/analytics-alert-control-plane/v1/resource/{resourceType}/id/{resourceId}/alerts",
        ))
        .path_param("resourceType", resource_type)
        .path_param("resourceId", resource_id)
        .json(&input)?;
        self.transport.execute(request).await
    }

    pub async fn update_alert(
        &self,
        resource_type: &str,
        resource_id: &str,
        alert_id: &str,
        input: UpdateAlertInput,
    ) -> Result<Value, Error> {
        let request = Request::new(Operation::patch(Server::Apis, "/analytics-alert-control-plane/v1/resource/{resourceType}/id/{resourceId}/alerts/{alertId}"))
            .path_param("resourceType", resource_type).path_param("resourceId", resource_id).path_param("alertId", alert_id).json(&input)?;
        self.transport.execute(request).await
    }
}
