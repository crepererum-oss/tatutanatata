use anyhow::{Context, Result};
use clap::Parser;
use tracing::debug;

use crate::{
    client::Client,
    non_empty_string::NonEmptyString,
    proto::{SaltServiceRequest, SaltServiceResponse},
};

/// Login CLI config.
#[derive(Debug, Parser)]
pub struct LoginCLIConfig {
    /// Username
    #[clap(long, env = "TUTANOTA_CLI_USERNAME")]
    username: NonEmptyString,

    /// Password
    #[clap(long, env = "TUTANOTA_CLI_PASSWORD")]
    password: NonEmptyString,
}

/// Perform tutanota webinterface login.
pub async fn perform_login(config: LoginCLIConfig, client: &Client) -> Result<()> {
    debug!("perform login");

    let req = SaltServiceRequest {
        format: Default::default(),
        mail_address: config.username.to_string(),
    };
    let salt: SaltServiceResponse = client
        .service_requst("saltservice", &req)
        .await
        .context("get salt")?;

    Ok(())
}
