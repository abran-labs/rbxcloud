use clap::{Args, Subcommand};
use rbxcloud::rbx::credentials::Credentials;

#[derive(Debug, Args)]
pub struct Auth {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Check for a locally usable API key; does not contact Roblox
    Status,
}

impl Auth {
    pub fn run(self) -> anyhow::Result<Option<String>> {
        match self.command {
            Command::Status => {
                Credentials::from_process()?;
                Ok(Some(
                    "Open Cloud API key available locally (not verified with Roblox)".into(),
                ))
            }
        }
    }
}
