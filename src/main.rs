mod cli;

#[cfg(test)]
#[path = "../tests/support/mod.rs"]
mod test_support;

use clap::{CommandFactory, FromArgMatches};
use cli::Cli;
use std::process;

fn hide_credential_env_values(command: clap::Command) -> clap::Command {
    let mut command = command.mut_args(|arg| match arg.get_env().and_then(|env| env.to_str()) {
        Some(
            rbxcloud::rbx::credentials::API_KEY_VAR
            | rbxcloud::rbx::credentials::LEGACY_API_KEY_VAR,
        ) => arg.hide_env_values(true),
        _ => arg,
    });
    for subcommand in command.get_subcommands_mut() {
        *subcommand = hide_credential_env_values(std::mem::take(subcommand));
    }
    command
}

#[tokio::main]
async fn main() {
    let mut matches = hide_credential_env_values(Cli::command()).get_matches();
    let cli_args = Cli::from_arg_matches_mut(&mut matches).unwrap_or_else(|error| error.exit());

    match cli_args.run().await {
        Ok(str) => {
            if let Some(s) = str {
                println!("{s}");
            }
        }
        Err(err) => {
            eprintln!("{err:?}");
            process::exit(1);
        }
    }
}
