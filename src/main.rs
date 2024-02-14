use std::path::PathBuf;

use crate::{
    client::Client,
    mails::Mail,
    session::{LoginCLIConfig, Session},
};
use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use folders::Folder;
use futures::TryStreamExt;
use logging::{setup_logging, LoggingCLIConfig};
use tracing::{debug, info};

mod blob;
mod client;
mod constants;
mod crypto;
mod folders;
mod logging;
mod mails;
mod non_empty_string;
mod proto;
mod session;

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

#[derive(Debug, Parser)]
struct DownloadCLIConfig {
    /// Folder name.
    #[clap(long, action)]
    folder: String,

    /// Target path.
    #[clap(long, action)]
    path: PathBuf,
}

/// Command
#[derive(Debug, Subcommand)]
enum Command {
    /// List folders.
    ListFolders,

    /// Download emails for given folder.
    Download(DownloadCLIConfig),
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    let args = Args::parse();
    setup_logging(args.logging_cfg).context("logging setup")?;

    let client = Client::try_new().context("set up client")?;

    let session = Session::login(args.login_cfg, &client)
        .await
        .context("perform login")?;

    let cmd_res = exec_cmd(&client, &session, args.command)
        .await
        .context("execute command");
    let logout_res = session.logout(&client).await.context("logout");

    match (cmd_res, logout_res) {
        (Err(e), _) => Err(e),
        (_, Err(e)) => Err(e),
        (Ok(()), Ok(())) => Ok(()),
    }
}

async fn exec_cmd(client: &Client, session: &Session, cmd: Command) -> Result<()> {
    match cmd {
        Command::ListFolders => {
            let folders = Folder::list(client, session).await.context("get folders")?;
            let mut folders = std::pin::pin!(folders);

            while let Some(f) = folders.try_next().await.context("poll folder")? {
                println!("{}", f.name);
            }

            Ok(())
        }
        Command::Download(cfg) => {
            // ensure output exists
            tokio::fs::create_dir_all(&cfg.path)
                .await
                .context("create output dir")?;

            // find folder
            let folders = Folder::list(client, session)
                .await
                .context("get folders")?
                .try_filter(|f| futures::future::ready(f.name == cfg.folder));
            let mut folders = std::pin::pin!(folders);
            let folder = folders
                .try_next()
                .await
                .context("search folder")?
                .context("folder not found")?;
            debug!(mails = folder.mails.as_str(), "download mails from folder");

            let mails = Mail::list(client, session, &folder);
            let mut mails = std::pin::pin!(mails);
            while let Some(mail) = mails.try_next().await.context("list mails")? {
                let target_file = cfg.path.join(&mail.mail_id).with_extension(".eml");
                if tokio::fs::try_exists(&target_file)
                    .await
                    .context("check file existence")?
                {
                    info!(id = mail.mail_id.as_str(), "already exists");
                } else {
                    info!(id = mail.mail_id.as_str(), "download");
                    mail.download(client, session, &target_file)
                        .await
                        .context("download mail")?;
                }
            }

            Ok(())
        }
    }
}
