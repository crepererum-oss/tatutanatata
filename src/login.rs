use anyhow::{bail, Context, Result};
use clap::Parser;
use reqwest::Method;
use tracing::debug;

use crate::{
    client::Client,
    crypto::auth::build_auth_verifier,
    non_empty_string::NonEmptyString,
    proto::{
        SaltServiceRequest, SaltServiceResponse, SessionServiceRequest, SessionServiceResponse,
    },
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

/// User session
#[derive(Debug)]
pub struct Session {
    pub user_id: String,
    pub access_token: String,
}

/// Perform tutanota login.
pub async fn perform_login(config: LoginCLIConfig, client: &Client) -> Result<Session> {
    debug!("perform login");

    let req = SaltServiceRequest {
        format: Default::default(),
        mail_address: config.username.to_string(),
    };
    let resp: SaltServiceResponse = client
        .service_requst(Method::GET, "saltservice", &req, None)
        .await
        .context("get salt")?;

    let auth_verifier = build_auth_verifier(resp.kdf_version, &config.password, resp.salt.as_ref())
        .context("build auth verifier")?;

    let req = SessionServiceRequest {
        format: Default::default(),
        access_key: Default::default(),
        auth_token: Default::default(),
        auth_verifier,
        client_identifier: env!("CARGO_PKG_NAME").to_owned(),
        mail_address: config.username.to_string(),
        recover_code_verifier: Default::default(),
        user: Default::default(),
    };
    let resp: SessionServiceResponse = client
        .service_requst(Method::POST, "sessionservice", &req, None)
        .await
        .context("get session")?;

    debug!(user = resp.user.as_str(), "got user");

    if !resp.challenges.is_empty() {
        bail!("not implemented: challenges");
    }

    Ok(Session {
        user_id: resp.user,
        access_token: resp.access_token,
    })
}
