use std::{path::PathBuf, sync::Arc};

use crate::{
    client::Client,
    eml::emit_eml,
    file_output::{escape_file_string, write_to_file},
    mails::Mail,
    session::{LoginCLIConfig, Session},
};
use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use folders::Folder;
use futures::{StreamExt, TryStreamExt};
use logging::{setup_logging, LoggingCLIConfig};
use tracing::{debug, info};

// Workaround for "unused crate" lint false positives.
#[cfg(test)]
use assert_cmd as _;
#[cfg(test)]
use similar_asserts as _;
#[cfg(test)]
use tempfile as _;

mod blob;
mod client;
mod compression;
mod constants;
mod crypto;
mod eml;
mod file_output;
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
    /// Concurrent downloads.
    #[clap(long, action, default_value_t = 5)]
    concurrent_downloads: usize,

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

            Mail::list(client, session, &folder)
                .map(|mail| {
                    let cfg = &cfg;

                    async move {
                        let mail = mail.context("list mail")?;

                        let target_file = cfg.path.join(format!(
                            "{}-{}.eml",
                            mail.date.format("%Y-%m-%d-%Hh%Mm%Ss"),
                            escape_file_string(&mail.subject)
                                .chars()
                                .take(64)
                                .collect::<String>(),
                        ));

                        if tokio::fs::try_exists(&target_file)
                            .await
                            .context("check file existence")?
                        {
                            info!(
                                folder_id = mail.folder_id.as_str(),
                                mail_id = mail.mail_id.as_str(),
                                target_file = %target_file.display(),
                                ui_url = mail.ui_url().as_str(),
                                "already exists",
                            );
                        } else {
                            info!(
                                folder_id = mail.folder_id.as_str(),
                                mail_id = mail.mail_id.as_str(),
                                target_file = %target_file.display(),
                                ui_url = mail.ui_url().as_str(),
                                "download",
                            );

                            let mail = Arc::clone(&mail)
                                .download(client, session)
                                .await
                                .with_context(|| format!("download mail: `{}`", mail.ui_url()))?;

                            let eml = emit_eml(&mail)
                                .with_context(|| format!("emit eml: `{}`", mail.mail.ui_url()))?;
                            write_to_file(eml.as_bytes(), &target_file)
                                .await
                                .with_context(|| {
                                    format!("write output file: `{}`", target_file.display())
                                })?;
                        }

                        Ok(()) as Result<()>
                    }
                })
                .buffer_unordered(cfg.concurrent_downloads)
                .try_collect::<()>()
                .await?;

            Ok(())
        }
    }
}
