use anyhow::{bail, ensure, Result};
use clap::Parser;
use rbxcloud::rbx::{
    client::{AsyncClient, Error},
    generated::assets::{
        AssetCloudClient, GetAssetVersionParams, ListAssetQuotasParams, ListAssetVersionsParams,
    },
};
use serde_json::Value;

use crate::cli::{asset_batch::BatchArgs, assets_cli::AssetsCommands, Cli, Command};

#[derive(Clone, Copy)]
pub(super) enum Target {
    GetVersion,
    ListVersions,
    ListQuotas,
    UploadImages,
}

pub(super) enum Action {
    GetVersion(GetAssetVersionParams),
    ListVersions(ListAssetVersionsParams),
    ListQuotas(ListAssetQuotasParams),
    Upload(BatchArgs),
}

const CREATOR_TYPE: &str = "--creator-type";
const CREATOR_ID: &str = "--creator-id";
const ASSET_TYPE: &str = "--asset-type";
const ASSET_ID: &str = "--asset-id";
const VERSION: &str = "--version-number";
const USER_ID: &str = "--user-id";
const MAX_PATHS: usize = 128;

impl Target {
    pub fn select(command: &[String]) -> Result<Self> {
        let [family, verb] = command else {
            bail!("expected one literal command family and subcommand")
        };
        Ok(match (family.as_str(), verb.as_str()) {
            ("asset-tools", "get-asset-version") => Self::GetVersion,
            ("asset-tools", "list-asset-versions") => Self::ListVersions,
            ("asset-tools", "list-asset-quotas") => Self::ListQuotas,
            ("assets", "upload-batch") => Self::UploadImages,
            _ => bail!("command target is not allowlisted"),
        })
    }

    pub fn is_read(self) -> bool {
        !matches!(self, Self::UploadImages)
    }

    pub fn path_slot(self, index: usize) -> bool {
        matches!(self, Self::UploadImages) && index >= 7
    }

    pub fn layout(self, args: &[String]) -> Result<()> {
        let args: Vec<&str> = args.iter().map(String::as_str).collect();
        let ok = match self {
            Self::GetVersion => matches!(args.as_slice(), [ASSET_ID, _, VERSION, _]),
            Self::ListVersions => matches!(args.as_slice(), [ASSET_ID, _]),
            Self::ListQuotas => matches!(args.as_slice(), [USER_ID, _]),
            Self::UploadImages => {
                matches!(
                    args.as_slice(),
                    [
                        CREATOR_TYPE,
                        _,
                        CREATOR_ID,
                        _,
                        ASSET_TYPE,
                        "image",
                        "--",
                        _,
                        ..
                    ]
                ) && args.len() <= 7 + MAX_PATHS
            }
        };
        ensure!(ok, "command arguments do not match the allowlisted layout");
        Ok(())
    }

    pub fn literal(self, index: usize, arg: &str) -> Result<()> {
        match (self, index) {
            (Self::GetVersion | Self::ListVersions | Self::ListQuotas, 1) => {
                positive::<i64>(arg)?;
            }
            (Self::GetVersion, 3) => {
                positive::<i32>(arg)?;
            }
            (Self::UploadImages, 1) => {
                ensure!(matches!(arg, "user" | "group"), "invalid creator type")
            }
            (Self::UploadImages, 3) => {
                positive::<u64>(arg)?;
            }
            (Self::UploadImages, index) if self.path_slot(index) => {
                use std::path::{Component, Path};
                ensure!(
                    !arg.is_empty()
                        && arg.len() <= 1024
                        && !arg.starts_with('-')
                        && !arg.chars().any(char::is_control)
                        && Path::new(arg)
                            .components()
                            .all(|part| matches!(part, Component::Normal(_) | Component::CurDir)),
                    "upload path must be a project-relative path, not a flag"
                );
            }
            _ => {}
        }
        Ok(())
    }

    pub fn build(self, command: &[String], args: Vec<String>) -> Result<Action> {
        self.layout(&args)?;
        for (index, arg) in args.iter().enumerate() {
            self.literal(index, arg)?;
        }
        // The narrow workflow grammar is independent of Clap defaults, aliases
        // and future flags. Clap then checks the *entire* final command in process.
        let cli = Cli::try_parse_from(
            std::iter::once("rbxcloud")
                .chain(command.iter().map(String::as_str))
                .chain(args.iter().map(String::as_str)),
        )
        .map_err(|_| anyhow::anyhow!("rendered arguments rejected by CLI parser"))?;
        Ok(match self {
            Self::GetVersion => Action::GetVersion(GetAssetVersionParams {
                asset_id: positive(&args[1])?,
                version_number: positive(&args[3])?,
            }),
            Self::ListVersions => Action::ListVersions(ListAssetVersionsParams {
                asset_id: positive(&args[1])?,
                max_page_size: None,
                page_token: None,
                filter: None,
            }),
            Self::ListQuotas => Action::ListQuotas(ListAssetQuotasParams {
                user_id: positive(&args[1])?,
                max_page_size: None,
                page_token: None,
                filter: None,
            }),
            Self::UploadImages => match cli.command {
                Command::Assets(assets) => match assets.command {
                    AssetsCommands::UploadBatch(batch) => Action::Upload(batch),
                    _ => bail!("rendered subcommand is not permitted"),
                },
                _ => bail!("rendered command family is not permitted"),
            },
        })
    }
}

fn positive<T: std::str::FromStr>(arg: &str) -> Result<T> {
    ensure!(
        !arg.is_empty() && !arg.starts_with('0') && arg.bytes().all(|b| b.is_ascii_digit()),
        "argument must be a positive integer"
    );
    arg.parse()
        .map_err(|_| anyhow::anyhow!("integer argument out of range"))
}

impl Action {
    pub async fn read(&self, transport: AsyncClient) -> Result<Value, Error> {
        let client = AssetCloudClient::new(transport);
        let result = match self {
            Self::GetVersion(params) => {
                serde_json::to_value(client.get_asset_version(params.clone()).await?)
            }
            Self::ListVersions(params) => {
                serde_json::to_value(client.list_asset_versions(params.clone()).await?)
            }
            Self::ListQuotas(params) => {
                serde_json::to_value(client.list_asset_quotas(params.clone()).await?)
            }
            Self::Upload(_) => unreachable!("not a read action"),
        };
        result.map_err(|_| Error::InvalidResponse {
            operation: "workflow read".into(),
            status: 200,
            reason: "cannot serialize output".into(),
        })
    }
}
