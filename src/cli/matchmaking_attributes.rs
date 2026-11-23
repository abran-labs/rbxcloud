use clap::{Args, Subcommand};
use rbxcloud::rbx::{
    client::{AsyncClient, ClientPolicy},
    credentials::Credentials,
    matchmaking_attributes::*,
};

#[derive(Debug, Args)]
pub struct PlayerAttributes {
    #[command(subcommand)]
    command: Command,
}
#[derive(Debug, Subcommand)]
enum Command {
    CreatePlayerAttribute(CreateArgs),
    UpdatePlayerAttribute(UpdateArgs),
}
#[derive(Debug, Args)]
struct ValueArgs {
    #[arg(long, conflicts_with_all = ["numeric_value", "string_value"])]
    bool_value: Option<bool>,
    #[arg(long, conflicts_with_all = ["bool_value", "string_value"], allow_negative_numbers = true)]
    numeric_value: Option<f64>,
    #[arg(long, conflicts_with_all = ["bool_value", "numeric_value"])]
    string_value: Option<String>,
    #[arg(long = "value-type")]
    value_type: Option<AttributeDataType>,
    #[arg(long)]
    data_store_name: Option<String>,
    #[arg(long)]
    data_store_scope: Option<String>,
    #[arg(long)]
    key_template: Option<String>,
    #[arg(long)]
    value_path: Option<String>,
}
impl ValueArgs {
    fn value(&self) -> Option<AttributeValue> {
        (self.bool_value.is_some()
            || self.numeric_value.is_some()
            || self.string_value.is_some()
            || self.value_type.is_some())
        .then_some(AttributeValue {
            bool_value: self.bool_value,
            numeric_value: self.numeric_value,
            string_value: self.string_value.clone(),
            value_type: self.value_type,
        })
    }
    fn location(&self) -> Option<AttributeLocation> {
        (self.data_store_name.is_some()
            || self.data_store_scope.is_some()
            || self.key_template.is_some()
            || self.value_path.is_some())
        .then_some(AttributeLocation {
            location_case: "DataStoreLocation".into(),
            data_store_location: DataStoreLocationInput {
                data_store_name: self.data_store_name.clone(),
                scope: self.data_store_scope.clone(),
                key_template: self.key_template.clone(),
                value_path: self.value_path.clone(),
            },
        })
    }
}
#[derive(Debug, Args)]
struct CreateArgs {
    #[arg(long)]
    universe_id: u64,
    #[arg(long)]
    name: Option<String>,
    #[arg(long)]
    data_type: Option<AttributeDataType>,
    #[command(flatten)]
    value: ValueArgs,
}
#[derive(Debug, Args)]
struct UpdateArgs {
    attribute_id: String,
    #[command(flatten)]
    value: ValueArgs,
}
impl PlayerAttributes {
    pub async fn run(self) -> anyhow::Result<Option<String>> {
        let client = MatchmakingAttributeClient::new(
            AsyncClient::new(Credentials::from_process()?, ClientPolicy::default()).await?,
        );
        let result = match self.command {
            Command::CreatePlayerAttribute(args) => {
                client
                    .create(CreatePlayerAttributeInput {
                        universe_id: args.universe_id,
                        name: args.name,
                        data_type: args.data_type,
                        default_value: args.value.value(),
                        attribute_value_location: args.value.location(),
                    })
                    .await?
            }
            Command::UpdatePlayerAttribute(args) => {
                client
                    .update(
                        args.attribute_id.clone(),
                        UpdatePlayerAttributeInput {
                            attribute_id: Some(args.attribute_id),
                            default_value: args.value.value(),
                            attribute_value_location: args.value.location(),
                        },
                    )
                    .await?
            }
        };
        Ok(Some(serde_json::to_string_pretty(&result)?))
    }
}
