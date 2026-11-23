use clap::{Args, Subcommand};
use rbxcloud::rbx::{
    client::{AsyncClient, ClientPolicy},
    credentials::Credentials,
    service_efficiency::*,
};

#[derive(Debug, Args)]
pub struct ServicePayments {
    #[command(subcommand)]
    command: Command,
}
#[derive(Debug, Subcommand)]
enum Command {
    UpdateGroupAccount {
        group_id: u64,
        #[command(flatten)]
        account: AccountArgs,
    },
    UpdateUserAccount {
        user_id: u64,
        #[command(flatten)]
        account: AccountArgs,
    },
    SubmitGroupPayment {
        group_id: u64,
        #[command(flatten)]
        payment: PaymentArgs,
    },
    SubmitUserPayment {
        user_id: u64,
        #[command(flatten)]
        payment: PaymentArgs,
    },
}
#[derive(Debug, Args)]
struct AccountArgs {
    #[arg(long)]
    account_tax_type: AccountTaxType,
    #[arg(long)]
    account_name: Option<String>,
    #[arg(long)]
    tax_id: Option<String>,
    #[arg(long)]
    tax_id_type: Option<TaxIdType>,
}
#[derive(Debug, Args)]
struct PaymentArgs {
    #[arg(long)]
    currency_code: Option<String>,
    #[arg(long, allow_negative_numbers = true)]
    units: Option<i64>,
    #[arg(long, allow_negative_numbers = true)]
    nanos: Option<i32>,
    #[arg(long)]
    bill_id: Option<String>,
}
impl ServicePayments {
    pub async fn run(self) -> anyhow::Result<Option<String>> {
        let transport =
            AsyncClient::new(Credentials::from_process()?, ClientPolicy::default()).await?;
        let client = ServiceEfficiencyClient::new(transport);
        let value = match self.command {
            Command::UpdateGroupAccount { group_id, account } => {
                client
                    .update_group_account(group_id, account.into())
                    .await?
            }
            Command::UpdateUserAccount { user_id, account } => {
                client.update_user_account(user_id, account.into()).await?
            }
            Command::SubmitGroupPayment { group_id, payment } => {
                client
                    .submit_group_payment(group_id, payment.into())
                    .await?
            }
            Command::SubmitUserPayment { user_id, payment } => {
                client.submit_user_payment(user_id, payment.into()).await?
            }
        };
        Ok(Some(serde_json::to_string_pretty(&value)?))
    }
}
impl From<AccountArgs> for AccountInput {
    fn from(value: AccountArgs) -> Self {
        Self {
            account_tax_type: value.account_tax_type,
            account_name: value.account_name,
            tax_id: value.tax_id,
            tax_id_type: value.tax_id_type,
        }
    }
}
impl From<PaymentArgs> for ManualPaymentInput {
    fn from(value: PaymentArgs) -> Self {
        let amount =
            (value.currency_code.is_some() || value.units.is_some() || value.nanos.is_some())
                .then_some(MoneyInput {
                    currency_code: value.currency_code,
                    units: value.units,
                    nanos: value.nanos,
                });
        Self {
            amount,
            bill_id: value.bill_id,
        }
    }
}
