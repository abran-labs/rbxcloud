use clap::{Args, Subcommand};
use rbxcloud::rbx::{
    asset_multipart::*,
    client::{AsyncClient, ClientPolicy},
    credentials::Credentials,
    generated::multipart_models::AssetsUploadApiCreator,
};

#[derive(Debug, Args)]
pub struct AssetMultipartStart {
    #[command(subcommand)]
    command: Command,
}
#[derive(Debug, Subcommand)]
enum Command {
    Create(UploadArgs),
    Update {
        asset_id: u64,
        #[command(flatten)]
        upload: UploadArgs,
    },
}
#[derive(Debug, Args)]
struct UploadArgs {
    #[arg(long)]
    display_name: String,
    #[arg(long, default_value = "")]
    description: String,
    #[arg(long)]
    asset_type: String,
    #[arg(long, conflicts_with_all = ["group_id", "experience_id"])]
    user_id: Option<String>,
    #[arg(long, conflicts_with_all = ["user_id", "experience_id"])]
    group_id: Option<String>,
    #[arg(long, conflicts_with_all = ["user_id", "group_id"])]
    experience_id: Option<String>,
    #[arg(long)]
    expected_price: Option<i64>,
    #[arg(long)]
    content_type: Option<String>,
    #[arg(long)]
    filesize: Option<i64>,
    #[arg(long)]
    md5_checksum: Option<String>,
    #[arg(long)]
    chunk_plan: Option<Vec<i64>>,
}
impl AssetMultipartStart {
    pub async fn run(self) -> anyhow::Result<Option<String>> {
        let client = AssetMultipartClient::new(
            AsyncClient::new(Credentials::from_process()?, ClientPolicy::default()).await?,
        );
        let result = match self.command {
            Command::Create(args) => client.create(args.into()).await?,
            Command::Update { asset_id, upload } => client.update(asset_id, upload.into()).await?,
        };
        Ok(Some(serde_json::to_string_pretty(&result)?))
    }
}
impl From<UploadArgs> for MultipartStartInput {
    fn from(value: UploadArgs) -> Self {
        let creator = AssetsUploadApiCreator {
            user_id: value.user_id,
            group_id: value.group_id,
            experience_id: value.experience_id,
        };
        Self {
            asset: asset_metadata(
                value.display_name,
                value.description,
                value.asset_type,
                creator,
                value.expected_price,
            ),
            file: UploadFileInput {
                content_type: value.content_type,
                filesize: value.filesize,
                md5_checksum: value.md5_checksum,
                chunk_plan: value.chunk_plan,
            },
        }
    }
}
