use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use logging::{setup_logging, LoggingCLIConfig};
use login::{perform_login, LoginCLIConfig};
use reqwest::Client;

mod logging;
mod login;
mod non_empty_string;

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

    let client = Client::builder().build().context("set up HTTPs client")?;

    perform_login(args.login_cfg, &client)
        .await
        .context("perform login")?;

    Ok(())
}
