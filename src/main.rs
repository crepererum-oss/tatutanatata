use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use commands::export::ExportCLIConfig;
use futures::FutureExt;
use logging::{setup_logging, LoggingCLIConfig};
use login::{perform_login, LoginCLIConfig};
use webdriver::{run_webdriver, WebdriverCLIConfig};

mod commands;
mod error;
mod logging;
mod login;
mod webdriver;

/// CLI args.
#[derive(Debug, Parser)]
struct Args {
    /// Logging config.
    #[clap(flatten)]
    logging_cfg: LoggingCLIConfig,

    /// Webdriver config.
    #[clap(flatten)]
    webdriver_cfg: WebdriverCLIConfig,

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

    /// Export emails
    Export(ExportCLIConfig),
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    let args = Args::parse();
    setup_logging(args.logging_cfg).context("logging setup")?;

    run_webdriver(args.webdriver_cfg, |webdriver| {
        async move {
            perform_login(args.login_cfg, webdriver)
                .await
                .context("perform login")?;

            match args.command {
                Command::ListFolders => {
                    let folders = commands::list_folders::list_folders(webdriver)
                        .await
                        .context("list folders")?;
                    for (_anchor, title) in folders {
                        println!("{title}");
                    }
                }
                Command::Export(config) => {
                    commands::export::export(config, webdriver)
                        .await
                        .context("export")?;
                }
            }

            Ok(())
        }
        .boxed()
    })
    .await
    .context("webdriver execution")?;

    Ok(())
}
