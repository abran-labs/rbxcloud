use std::path::PathBuf;

use clap::{Args, Subcommand};
use rbxcloud::rbx::{
    client::{AsyncClient, ClientPolicy},
    credentials::Credentials,
    game_localization::*,
};

#[derive(Debug, Args)]
pub struct GameLocalization {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    UploadBadgeIcon(ImageArgs),
    UploadDeveloperProductIcon(ImageArgs),
    UploadGameIcon(ImageArgs),
    UploadGamePassIcon(ImageArgs),
    UploadGameThumbnail(ImageArgs),
    UpdateNameDescriptions(NameDescriptionArgs),
    UpdateSupportedLanguages(LanguageArgs),
}

#[derive(Debug, Args)]
struct ImageArgs {
    id: u64,
    language_code: String,
    #[arg(long)]
    file: PathBuf,
    #[arg(long, default_value = "image/png")]
    content_type: String,
}

#[derive(Debug, Args)]
struct NameDescriptionArgs {
    game_id: u64,
    #[arg(long)]
    data_json: String,
}

#[derive(Debug, Args)]
struct LanguageArgs {
    game_id: u64,
    #[arg(long)]
    languages_json: String,
}

impl GameLocalization {
    pub async fn run(self) -> anyhow::Result<Option<String>> {
        let client = GameLocalizationClient::new(
            AsyncClient::new(Credentials::from_process()?, ClientPolicy::default()).await?,
        );
        let value = match self.command {
            Command::UploadBadgeIcon(args) => {
                client
                    .upload_badge_icon(args.id, &args.language_code, &args.file, &args.content_type)
                    .await?
            }
            Command::UploadDeveloperProductIcon(args) => {
                client
                    .upload_developer_product_icon(
                        args.id,
                        &args.language_code,
                        &args.file,
                        &args.content_type,
                    )
                    .await?
            }
            Command::UploadGameIcon(args) => {
                client
                    .upload_game_icon(args.id, &args.language_code, &args.file, &args.content_type)
                    .await?
            }
            Command::UploadGamePassIcon(args) => {
                client
                    .upload_game_pass_icon(
                        args.id,
                        &args.language_code,
                        &args.file,
                        &args.content_type,
                    )
                    .await?
            }
            Command::UploadGameThumbnail(args) => {
                client
                    .upload_game_thumbnail(
                        args.id,
                        &args.language_code,
                        &args.file,
                        &args.content_type,
                    )
                    .await?
            }
            Command::UpdateNameDescriptions(args) => {
                let data = serde_json::from_str::<Vec<NameDescriptionInput>>(&args.data_json)?;
                client.update_name_descriptions(args.game_id, data).await?
            }
            Command::UpdateSupportedLanguages(args) => {
                let languages =
                    serde_json::from_str::<Vec<PatchLanguageInput>>(&args.languages_json)?;
                client
                    .update_supported_languages(args.game_id, languages)
                    .await?
            }
        };
        Ok(Some(serde_json::to_string_pretty(&value)?))
    }
}
