use std::collections::HashMap;

use anyhow::{bail, Context, Result};
use clap::Parser;
use reqwest::Method;
use sha2::{Digest, Sha256};
use tracing::debug;

use crate::{
    client::Client,
    constants::APP_USER_AGENT,
    crypto::{
        auth::{derive_passkey, encode_auth_verifier},
        encryption::decrypt_key,
    },
    non_empty_string::NonEmptyString,
    proto::{
        Base64Url, SaltServiceRequest, SaltServiceResponse, SessionServiceRequest,
        SessionServiceResponse, UserResponse,
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
    pub access_token: Base64Url,
    pub group_keys: HashMap<String, Vec<u8>>,
    pub user_data: UserResponse,
}

impl Session {
    /// Perform tutanota login.
    pub async fn login(config: LoginCLIConfig, client: &Client) -> Result<Self> {
        debug!("perform login");

        let req = SaltServiceRequest {
            format: Default::default(),
            mail_address: config.username.to_string(),
        };
        let resp: SaltServiceResponse = client
            .service_request(Method::GET, "saltservice", &req, None)
            .await
            .context("get salt")?;

        let pk = derive_passkey(resp.kdf_version, &config.password, resp.salt.as_ref())
            .context("derive passkey")?;
        let auth_verifier = encode_auth_verifier(&pk);

        let req = SessionServiceRequest {
            format: Default::default(),
            access_key: Default::default(),
            auth_token: Default::default(),
            auth_verifier,
            client_identifier: APP_USER_AGENT.to_owned(),
            mail_address: config.username.to_string(),
            recover_code_verifier: Default::default(),
            user: Default::default(),
        };
        let resp: SessionServiceResponse = client
            .service_request(Method::POST, "sessionservice", &req, None)
            .await
            .context("get session")?;
        let user_id = resp.user;
        let access_token = resp.access_token;

        debug!(user = user_id.as_str(), "got user");

        if !resp.challenges.is_empty() {
            bail!("not implemented: challenges");
        }

        let user_data: UserResponse = client
            .service_request(
                Method::GET,
                &format!("user/{}", user_id),
                &(),
                Some(&access_token),
            )
            .await
            .context("get user")?;

        let user_key = decrypt_key(&pk, user_data.user_group.sym_enc_g_key.as_ref())
            .context("decrypt user group key")?;
        let mut group_keys = HashMap::default();
        group_keys.insert(user_data.user_group.group.clone(), user_key.clone());
        for group in &user_data.memberships {
            group_keys.insert(
                group.group.clone(),
                decrypt_key(&user_key, group.sym_enc_g_key.as_ref())
                    .context("decrypt membership group key")?,
            );
        }

        Ok(Self {
            user_id,
            access_token,
            group_keys,
            user_data,
        })
    }

    pub async fn logout(self, client: &Client) -> Result<()> {
        let session = &self.user_data.auth.sessions;

        debug!(session = session.as_str(), "performing logout",);

        client
            .service_request_no_response(
                Method::DELETE,
                &format!(
                    "session/{}/{}",
                    session,
                    session_element_id(&self.access_token)
                ),
                &(),
                Some(&self.access_token),
            )
            .await
            .context("session deletion")?;

        debug!("logout done");

        Ok(())
    }
}

const GENERATE_ID_BYTES_LENGTH: usize = 9;

fn session_element_id(access_token: &Base64Url) -> Base64Url {
    let mut hasher = Sha256::new();
    hasher.update(&access_token.as_ref()[GENERATE_ID_BYTES_LENGTH..]);
    hasher.finalize().to_vec().into()
}

#[allow(dead_code)]
fn session_list_id(access_token: &Base64Url) -> Base64Url {
    access_token.as_ref()[..GENERATE_ID_BYTES_LENGTH].into()
}
