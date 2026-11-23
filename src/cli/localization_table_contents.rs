use clap::{Args, Subcommand};
use rbxcloud::rbx::{
    client::{AsyncClient, ClientPolicy},
    credentials::Credentials,
    localization_table_contents::*,
};

#[derive(Debug, Args)]
pub struct LocalizationTableContents {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    UpdateTable(UpdateArgs),
    TranslationHistory(HistoryArgs),
}

#[derive(Debug, Args)]
struct UpdateArgs {
    table_id: String,
    #[arg(long)]
    name: Option<String>,
    #[arg(long)]
    entries_json: String,
}

#[derive(Debug, Args)]
struct HistoryArgs {
    table_id: String,
    #[arg(long)]
    locale: Option<String>,
    #[arg(long)]
    entries_json: String,
    #[arg(long)]
    source_type: Option<TranslationSourceType>,
}

impl LocalizationTableContents {
    pub async fn run(self) -> anyhow::Result<Option<String>> {
        let client = LocalizationTableContentClient::new(
            AsyncClient::new(Credentials::from_process()?, ClientPolicy::default()).await?,
        );
        let value = match self.command {
            Command::UpdateTable(args) => {
                let entries = serde_json::from_str::<Vec<PatchEntryInput>>(&args.entries_json)?;
                client
                    .update_table(
                        args.table_id,
                        UpdateTableContentsInput {
                            name: args.name,
                            entries,
                        },
                    )
                    .await?
            }
            Command::TranslationHistory(args) => {
                let entries =
                    serde_json::from_str::<Vec<CursorEntryIdentifierInput>>(&args.entries_json)?;
                client
                    .translation_history(
                        args.table_id,
                        TranslationHistoryInput {
                            locale: args.locale,
                            entries,
                            source_type: args.source_type,
                        },
                    )
                    .await?
            }
        };
        Ok(Some(serde_json::to_string_pretty(&value)?))
    }
}
