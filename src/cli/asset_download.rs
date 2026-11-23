use clap::{Args, Subcommand};
use rbxcloud::rbx::{
    asset_delivery::*,
    client::{AsyncClient, ClientPolicy},
    credentials::Credentials,
};
use std::path::PathBuf;

#[derive(Debug, Args)]
pub struct AssetDownload {
    #[command(subcommand)]
    command: Command,
}
#[derive(Debug, Subcommand)]
enum Command {
    GetAsset(DownloadArgs),
    GetAssetVersion(DownloadArgs),
}
#[derive(Debug, Args)]
struct DownloadArgs {
    asset_id: u64,
    #[arg(long)]
    version_number: Option<u32>,
    #[arg(long)]
    output: PathBuf,
    #[arg(long)]
    accept_encoding: Option<String>,
    #[arg(long)]
    place_id: Option<u64>,
    #[arg(long)]
    asset_type: Option<String>,
    #[arg(long)]
    asset_format: Option<String>,
    #[arg(long)]
    roblox_asset_format: Option<String>,
    #[arg(long)]
    usage_context: Option<String>,
}
impl AssetDownload {
    pub async fn run(self) -> anyhow::Result<Option<String>> {
        let args = match self.command {
            Command::GetAsset(args) | Command::GetAssetVersion(args) => args,
        };
        let client = AssetDeliveryClient::new(
            AsyncClient::new(Credentials::from_process()?, ClientPolicy::default()).await?,
        );
        let result = client
            .download(
                AssetDownloadInput {
                    asset_id: args.asset_id,
                    version_number: args.version_number,
                    accept_encoding: args.accept_encoding,
                    place_id: args.place_id,
                    asset_type: args.asset_type,
                    asset_format: args.asset_format,
                    roblox_asset_format: args.roblox_asset_format,
                    usage_context: args.usage_context,
                },
                &args.output,
            )
            .await?;
        Ok(Some(serde_json::json!({"path": result.path, "contentType": result.content_type, "byteLength": result.byte_length}).to_string()))
    }
}
