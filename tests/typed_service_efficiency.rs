mod support;
use rbxcloud::rbx::{
    client::{AsyncClient, ClientPolicy},
    credentials::{Credentials, SecretString},
    service_efficiency::*,
};
use std::time::Duration;
use support::{Reply, TestServer};

async fn client(server: &TestServer) -> ServiceEfficiencyClient {
    let policy = ClientPolicy::loopback(server.addr())
        .unwrap()
        .timeout(Duration::from_secs(2))
        .retry_delay(Duration::from_millis(1));
    ServiceEfficiencyClient::new(
        AsyncClient::new(
            Credentials::api_key(SecretString::new("service-contract-test-key")),
            policy,
        )
        .await
        .unwrap(),
    )
}

#[tokio::test]
async fn nested_account_and_money_fields_serialize_exactly() {
    let server = TestServer::start(vec![Reply::json(200, "{}"), Reply::json(200, "{}")]);
    let client = client(&server).await;
    client
        .update_user_account(
            5,
            AccountInput {
                account_tax_type: AccountTaxType::Business,
                account_name: Some("Abran Labs".into()),
                tax_id: Some("tax".into()),
                tax_id_type: Some(TaxIdType::CanadianBusinessNumber),
            },
        )
        .await
        .unwrap();
    client
        .submit_user_payment(
            5,
            ManualPaymentInput {
                amount: Some(MoneyInput {
                    currency_code: Some("CAD".into()),
                    units: Some(12),
                    nanos: Some(500_000_000),
                }),
                bill_id: Some("bill".into()),
            },
        )
        .await
        .unwrap();
    let requests = server.requests();
    assert_eq!(
        requests[0].target,
        "/service-efficiency-api/v1/accounts/users/5"
    );
    assert_eq!(
        requests[0].body_text(),
        r#"{"account":{"accountTaxType":"Business","accountName":"Abran Labs","taxId":"tax","taxIdType":"CanadianBusinessNumber"}}"#
    );
    assert_eq!(
        requests[1].body_text(),
        r#"{"amount":{"currencyCode":"CAD","units":12,"nanos":500000000},"billId":"bill"}"#
    );
}
