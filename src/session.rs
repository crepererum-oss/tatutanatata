use std::{collections::HashMap, ops::Deref, sync::Arc};

use anyhow::{bail, Context, Result};
use clap::Parser;
use reqwest::Method;
use sha2::{Digest, Sha256};
use tracing::debug;

use crate::{
    client::{Client, Prefix, Request, DEFAULT_HOST},
    constants::APP_USER_AGENT,
    crypto::{
        auth::{derive_passkey, encode_auth_verifier, UserPassphraseKey},
        encryption::decrypt_key,
    },
    non_empty_string::NonEmptyString,
    proto::{
        binary::Base64Url,
        keys::Key,
        messages::{
            SaltServiceRequest, SaltServiceResponse, SessionServiceRequest, SessionServiceResponse,
            UserResponse,
        },
    },
};

/// Login CLI config.
#[derive(Debug, Parser)]
pub(crate) struct LoginCLIConfig {
    /// Username
    #[clap(long, env = "TUTANOTA_CLI_USERNAME")]
    username: NonEmptyString,

    /// Password
    #[clap(long, env = "TUTANOTA_CLI_PASSWORD")]
    password: NonEmptyString,
}

/// User session
#[derive(Debug)]
pub(crate) struct Session {
    #[allow(dead_code)]
    pub(crate) user_id: String,
    pub(crate) access_token: Base64Url,
    pub(crate) group_keys: Arc<GroupKeys>,
    pub(crate) user_data: UserResponse,
}

impl Session {
    /// Perform tutanota login.
    pub(crate) async fn login(config: LoginCLIConfig, client: &Client) -> Result<Self> {
        debug!("perform login");

        let req = SaltServiceRequest {
            format: Default::default(),
            mail_address: config.username.to_string(),
        };
        let resp: SaltServiceResponse = client
            .do_json(Request::new(Prefix::Sys, "saltservice", &req))
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
            .do_json(Request {
                method: Method::POST,
                ..Request::new(Prefix::Sys, "sessionservice", &req)
            })
            .await
            .context("get session")?;
        let user_id = resp.user;
        let access_token = resp.access_token;

        debug!(user = user_id.as_str(), "got user");

        if !resp.challenges.is_empty() {
            bail!("not implemented: challenges");
        }

        let user_data: UserResponse = client
            .do_json(Request {
                access_token: Some(&access_token),
                ..Request::new(Prefix::Sys, &format!("user/{}", user_id), &())
            })
            .await
            .context("get user")?;

        let group_keys =
            Arc::new(GroupKeys::try_new(&pk, &user_data).context("set up group keys")?);

        Ok(Self {
            user_id,
            access_token,
            group_keys,
            user_data,
        })
    }

    pub(crate) async fn logout(self, client: &Client) -> Result<()> {
        let session = &self.user_data.auth.sessions;

        debug!(session = session.as_str(), "performing logout",);

        client
            .do_no_response(Request {
                method: Method::DELETE,
                host: DEFAULT_HOST,
                prefix: Prefix::Sys,
                path: &format!(
                    "session/{}/{}",
                    session,
                    session_element_id(&self.access_token)
                ),
                data: &(),
                access_token: Some(&self.access_token),
                query: &[],
            })
            .await
            .context("session deletion")?;

        debug!("logout done");

        Ok(())
    }
}

#[derive(Debug)]
pub(crate) struct GroupKeys {
    keys: HashMap<String, Key>,
}

impl GroupKeys {
    fn try_new(pk: &UserPassphraseKey, user_data: &UserResponse) -> Result<Self> {
        let user_key = decrypt_key(
            *pk.deref(),
            user_data
                .user_group
                .sym_enc_g_key
                .0
                .context("user key must be set")?,
        )
        .context("decrypt user group key")?;
        let mut group_keys = HashMap::default();
        group_keys.insert(user_data.user_group.group.clone(), user_key);
        for group in &user_data.memberships {
            if let Some(enc_g_key) = group.sym_enc_g_key.0 {
                group_keys.insert(
                    group.group.clone(),
                    decrypt_key(user_key, enc_g_key).context("decrypt membership group key")?,
                );
            }
        }

        Ok(Self { keys: group_keys })
    }

    pub(crate) fn get(&self, group: &str) -> Result<Key> {
        let key = self.keys.get(group).context("group key not found")?;
        Ok(*key)
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
