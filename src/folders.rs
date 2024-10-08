use std::{
    collections::{hash_map::Entry, HashMap},
    sync::Arc,
};

use anyhow::{bail, Context, Result};
use futures::{Stream, TryStreamExt};
use reqwest::Method;
use tracing::debug;

use crate::{
    client::{Client, Prefix, Request, DEFAULT_HOST},
    crypto::encryption::{decrypt_key, decrypt_value},
    proto::{
        enums::{GroupType, MailFolderType},
        messages::{FolderResponse, MailboxGroupRootResponse, MailboxResponse, UserMembership},
    },
    session::{GroupKeys, Session},
};

#[derive(Debug)]
pub(crate) struct Folder {
    pub(crate) name: String,
    pub(crate) mails: String,
    pub(crate) id: String,
}

impl Folder {
    pub(crate) async fn list(
        client: &Client,
        session: &Session,
    ) -> Result<impl Stream<Item = Result<Self>>> {
        let mail_group = get_mail_membership(session).context("get mail group")?;

        let resp: MailboxGroupRootResponse = client
            .do_json(Request {
                method: Method::GET,
                host: DEFAULT_HOST,
                prefix: Prefix::Tutanota,
                path: &format!("mailboxgrouproot/{}", mail_group.group),
                data: &(),
                access_token: Some(&session.access_token),
                query: &[],
            })
            .await
            .context("get mailbox group root")?;
        let mailbox = resp.mailbox;

        debug!(mailbox = mailbox.as_str(), "mailbox found");

        let resp: MailboxResponse = client
            .do_json(Request {
                method: Method::GET,
                host: DEFAULT_HOST,
                prefix: Prefix::Tutanota,
                path: &format!("mailbox/{mailbox}"),
                data: &(),
                access_token: Some(&session.access_token),
                query: &[],
            })
            .await
            .context("get mailbox")?;
        let folders = resp.folders.folders;

        debug!(folders = folders.as_str(), "folders found");

        let group_keys = Arc::clone(&session.group_keys);
        let stream = client
            .stream::<FolderResponse>(
                &format!("mailfolder/{folders}"),
                Some(&session.access_token),
            )
            .and_then(move |f| {
                let group_keys = Arc::clone(&group_keys);
                async move { Self::decode(f, &group_keys) }
            });

        Ok(stream)
    }

    fn decode(resp: FolderResponse, group_keys: &GroupKeys) -> Result<Self> {
        let session_key = decrypt_key(
            group_keys
                .get(&resp.owner_group)
                .context("getting owner group key")?,
            resp.owner_enc_session_key,
        )
        .context("decrypting session key")?;

        let name = if resp.folder_type == MailFolderType::Custom {
            String::from_utf8(
                decrypt_value(session_key, resp.name.as_ref()).context("decrypt folder name")?,
            )
            .context("invalid UTF8 string")?
        } else {
            resp.folder_type.name().to_owned()
        };

        Ok(Self {
            name,
            mails: resp.mails,
            id: resp.id[1].clone(),
        })
    }
}

fn get_mail_membership(session: &Session) -> Result<UserMembership> {
    debug!("get mail membership");

    let mut memberships = HashMap::with_capacity(session.user_data.memberships.len());
    for membership in &session.user_data.memberships {
        match memberships.entry(membership.group_type) {
            Entry::Vacant(v) => {
                v.insert(membership);
            }
            Entry::Occupied(_) => bail!(
                "duplicate group membership for type {:?}",
                membership.group_type
            ),
        }
    }

    let membership = *memberships
        .get(&GroupType::Mail)
        .context("no mail group found")?;

    debug!(group = membership.group.as_str(), "got mail membership");

    Ok(membership.clone())
}
