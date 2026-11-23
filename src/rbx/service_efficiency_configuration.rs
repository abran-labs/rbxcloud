//! Typed service-efficiency consumer configuration updates.

use super::client::{AsyncClient, Error, Operation, Request, Server};
use serde::{Deserialize, Serialize};
use serde_json::Value;
#[derive(Debug, Clone, Copy, clap::ValueEnum, Serialize, Deserialize)]
pub enum ServiceBudgetLevel {
    ResourceOnly,
    ServiceOnly,
}
impl std::fmt::Display for ServiceBudgetLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ServiceMoneyInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub units: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nanos: Option<i32>,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ServicePriceInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost: Option<ServiceMoneyInput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit_amount: Option<i64>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ResourceConfigurationInput {
    pub resource_id: String,
    pub unlocked: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub monthly_budget: Option<ServiceMoneyInput>,
    pub unit_cost: ServiceMoneyInput,
    pub price: ServicePriceInput,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ServiceConfigurationInput {
    pub service_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub monthly_budget: Option<ServiceMoneyInput>,
    pub resource_configurations: Vec<ResourceConfigurationInput>,
    pub budget_level: ServiceBudgetLevel,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ConfigurationBody {
    service_configurations: Vec<ServiceConfigurationInput>,
}
#[derive(Debug, Clone)]
pub struct ServiceEfficiencyClient {
    transport: AsyncClient,
}
impl ServiceEfficiencyClient {
    pub fn new(transport: AsyncClient) -> Self {
        Self { transport }
    }
    pub async fn update(
        &self,
        universe_id: u64,
        configurations: Vec<ServiceConfigurationInput>,
    ) -> Result<Value, Error> {
        let request = Request::new(Operation::patch(
            Server::Apis,
            "/service-efficiency-api/v1/configurations/universes/{universeId}",
        ))
        .path_param("universeId", universe_id)
        .json(&ConfigurationBody {
            service_configurations: configurations,
        })?;
        self.transport.execute(request).await
    }
}
