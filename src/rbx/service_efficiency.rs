//! Typed service-efficiency account and payment operations.
use super::client::{AsyncClient, Error, Operation, Request, Server};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, clap::ValueEnum, Serialize, Deserialize)]
pub enum AccountTaxType {
    #[default]
    #[serde(rename = "Invalid")]
    Invalid,
    #[serde(rename = "Individual")]
    Individual,
    #[serde(rename = "Business")]
    Business,
}
impl std::fmt::Display for AccountTaxType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Invalid => "Invalid",
            Self::Individual => "Individual",
            Self::Business => "Business",
        })
    }
}

#[derive(Debug, Clone, Copy, clap::ValueEnum, Serialize, Deserialize)]
pub enum TaxIdType {
    #[serde(rename = "Invalid")]
    Invalid,
    #[serde(rename = "UnitedStatesEin")]
    UnitedStatesEin,
    #[serde(rename = "EuropeanVatNumber")]
    EuropeanVatNumber,
    #[serde(rename = "UnitedKingdomVatNumber")]
    UnitedKingdomVatNumber,
    #[serde(rename = "CanadianBusinessNumber")]
    CanadianBusinessNumber,
    #[serde(rename = "CanadianGstHstNumber")]
    CanadianGstHstNumber,
    #[serde(rename = "MexicanRfcNumber")]
    MexicanRfcNumber,
}
impl std::fmt::Display for TaxIdType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Invalid => "Invalid",
            Self::UnitedStatesEin => "UnitedStatesEin",
            Self::EuropeanVatNumber => "EuropeanVatNumber",
            Self::UnitedKingdomVatNumber => "UnitedKingdomVatNumber",
            Self::CanadianBusinessNumber => "CanadianBusinessNumber",
            Self::CanadianGstHstNumber => "CanadianGstHstNumber",
            Self::MexicanRfcNumber => "MexicanRfcNumber",
        })
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountInput {
    pub account_tax_type: AccountTaxType,
    pub account_name: Option<String>,
    pub tax_id: Option<String>,
    pub tax_id_type: Option<TaxIdType>,
}

#[derive(Debug, Clone, Default, Serialize)]
struct UpdateAccountRequest {
    account: AccountInput,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoneyInput {
    pub currency_code: Option<String>,
    pub units: Option<i64>,
    pub nanos: Option<i32>,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManualPaymentInput {
    pub amount: Option<MoneyInput>,
    pub bill_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ServiceEfficiencyClient {
    transport: AsyncClient,
}
impl ServiceEfficiencyClient {
    pub fn new(transport: AsyncClient) -> Self {
        Self { transport }
    }
    pub async fn update_group_account(
        &self,
        group_id: u64,
        account: AccountInput,
    ) -> Result<serde_json::Value, Error> {
        self.update_account(
            format!("/service-efficiency-api/v1/accounts/groups/{group_id}"),
            account,
        )
        .await
    }
    pub async fn update_user_account(
        &self,
        user_id: u64,
        account: AccountInput,
    ) -> Result<serde_json::Value, Error> {
        self.update_account(
            format!("/service-efficiency-api/v1/accounts/users/{user_id}"),
            account,
        )
        .await
    }
    async fn update_account(
        &self,
        path: String,
        account: AccountInput,
    ) -> Result<serde_json::Value, Error> {
        let template = if path.contains("/groups/") {
            "/service-efficiency-api/v1/accounts/groups/{id}"
        } else {
            "/service-efficiency-api/v1/accounts/users/{id}"
        };
        let id = path.rsplit('/').next().unwrap_or_default();
        let request = Request::new(Operation::put(Server::Apis, template))
            .path_param("id", id)
            .json(&UpdateAccountRequest { account })?;
        self.transport.execute(request).await
    }
    pub async fn submit_group_payment(
        &self,
        group_id: u64,
        payment: ManualPaymentInput,
    ) -> Result<serde_json::Value, Error> {
        self.submit_payment(
            "/service-efficiency-api/v1/payments/groups/{id}",
            group_id,
            payment,
        )
        .await
    }
    pub async fn submit_user_payment(
        &self,
        user_id: u64,
        payment: ManualPaymentInput,
    ) -> Result<serde_json::Value, Error> {
        self.submit_payment(
            "/service-efficiency-api/v1/payments/users/{id}",
            user_id,
            payment,
        )
        .await
    }
    async fn submit_payment(
        &self,
        template: &'static str,
        id: u64,
        payment: ManualPaymentInput,
    ) -> Result<serde_json::Value, Error> {
        let request = Request::new(Operation::post(Server::Apis, template))
            .path_param("id", id)
            .json(&payment)?;
        self.transport.execute(request).await
    }
}
