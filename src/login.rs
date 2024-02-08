use anyhow::{Context, Result};
use clap::Parser;
use reqwest::Client;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tracing::debug;

use crate::non_empty_string::NonEmptyString;

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
        format: "0".to_owned(),
        mail_address: config.username.to_string(),
    };
    let salt: SaltServiceResponse = service_requst(client, "saltservice", &req)
        .await
        .context("get salt")?;

    Ok(())
}

async fn service_requst<Req, Resp>(client: &Client, path: &str, req: &Req) -> Result<Resp>
where
    Req: serde::Serialize,
    Resp: DeserializeOwned,
{
    debug!(path, "service request",);

    let resp = client
        .get(format!("https://app.tuta.com/rest/sys/{path}"))
        .json(req)
        .send()
        .await
        .context("initial request")?
        .error_for_status()
        .context("return status")?
        .json::<Resp>()
        .await
        .context("fetch JSON response")?;

    Ok(resp)
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SaltServiceRequest {
    #[serde(rename = "_format")]
    format: String,

    mail_address: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SaltServiceResponse {
    #[serde(rename = "_format")]
    format: String,

    kdf_version: String,

    salt: String,
}
