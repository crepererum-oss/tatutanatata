use anyhow::{Context, Result};
use clap::Parser;
use reqwest::Method;
use sha2::{Digest, Sha256};
use tracing::debug;

use crate::{
    client::Client,
    non_empty_string::NonEmptyString,
    proto::{
        Base64String, KdfVersion, SaltServiceRequest, SaltServiceResponse, SessionServiceRequest,
        SessionServiceResponse,
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

/// Perform tutanota webinterface login.
pub async fn perform_login(config: LoginCLIConfig, client: &Client) -> Result<()> {
    debug!("perform login");

    let req = SaltServiceRequest {
        format: Default::default(),
        mail_address: config.username.to_string(),
    };
    let resp: SaltServiceResponse = client
        .service_requst(Method::GET, "saltservice", &req)
        .await
        .context("get salt")?;

    let passkey = derive_passkey(resp.kdf_version, &config.password, resp.salt.as_ref())
        .context("derive passkey")?;
    let auth_verifier = encode_auth_verifier(&passkey);

    let req = SessionServiceRequest {
        format: Default::default(),
        access_key: Default::default(),
        auth_token: Default::default(),
        auth_verifier,
        client_identifier: "test".to_owned(),
        mail_address: config.username.to_string(),
        recover_code_verifier: Default::default(),
        user: Default::default(),
    };
    let resp: SessionServiceResponse = client
        .service_requst(Method::POST, "sessionservice", &req)
        .await
        .context("get session")?;

    debug!(user = resp.user.as_str(), "got user");

    Ok(())
}

fn derive_passkey(kdf_version: KdfVersion, passphrase: &str, salt: &[u8]) -> Result<Vec<u8>> {
    match kdf_version {
        KdfVersion::Bcrypt => {
            let mut hasher = Sha256::new();
            hasher.update(passphrase.as_bytes());
            let passphrase = hasher.finalize();

            let salt: [u8; 16] = salt.try_into().context("salt length")?;

            let hashed = bcrypt::bcrypt(8, salt, &passphrase);

            let mut hasher = Sha256::new();
            hasher.update(&hashed[..16]);
            let res = hasher.finalize().to_vec();

            Ok(res)
        }
        KdfVersion::Argon2id => unimplemented!("Argon2id"),
    }
}

fn encode_auth_verifier(passkey: &[u8]) -> String {
    let base64 = Base64String::from(passkey).to_string();
    base64.replace('+', "-").replace('/', "_").replace('=', "")
}
