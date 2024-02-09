use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use folders::get_folders;
use logging::{setup_logging, LoggingCLIConfig};
use login::{perform_login, LoginCLIConfig};

mod client;
mod crypto;
mod folders;
mod logging;
mod login;
mod non_empty_string;
mod proto;

use client::Client;

/// CLI args.
#[derive(Debug, Parser)]
struct Args {
    /// Logging config.
    #[clap(flatten)]
    logging_cfg: LoggingCLIConfig,

    /// Login config.
    #[clap(flatten)]
    login_cfg: LoginCLIConfig,

    /// Command
    #[clap(subcommand)]
    command: Command,
}

/// Command
#[derive(Debug, Subcommand)]
enum Command {
    /// List folders.
    ListFolders,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    let args = Args::parse();
    setup_logging(args.logging_cfg).context("logging setup")?;

    let client = Client::try_new().context("set up client")?;

    let session = perform_login(args.login_cfg, &client)
        .await
        .context("perform login")?;

    match args.command {
        Command::ListFolders => {
            get_folders(&client, &session).await.context("get folders")?;
        }
    }

    Ok(())
}
