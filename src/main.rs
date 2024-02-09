use crate::{
    client::Client,
    session::{LoginCLIConfig, Session},
};
use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use folders::get_folders;
use logging::{setup_logging, LoggingCLIConfig};

mod client;
mod crypto;
mod folders;
mod logging;
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
            let folders = get_folders(client, session).await.context("get folders")?;
            for f in folders {
                println!("{}", f.name);
            }

            Ok(())
        }
    }
}
