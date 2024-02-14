use std::{
    collections::{hash_map::Entry, HashMap},
    sync::Arc,
};

use anyhow::{bail, Context, Result};
use futures::{Stream, TryStreamExt};
use reqwest::Method;
use tracing::debug;

use crate::{
    client::Client,
    crypto::encryption::{decrypt_key, decrypt_value},
    proto::{
        FolderResponse, GroupType, MailFolderType, MailboxGroupRootResponse, MailboxResponse,
        UserMembership,
    },
    session::Session,
};

#[derive(Debug)]
pub struct Folder {
    pub name: String,
    pub mails: String,
}

impl Folder {
    pub async fn list(
        client: &Client,
        session: &Session,
    ) -> Result<impl Stream<Item = Result<Folder>>> {
        let mail_group = get_mail_membership(session).context("get mail group")?;

        let resp: MailboxGroupRootResponse = client
            .service_request_tutanota(
                Method::GET,
                &format!("mailboxgrouproot/{}", mail_group.group),
                &(),
                Some(&session.access_token),
            )
            .await
            .context("get mailbox group root")?;
        let mailbox = resp.mailbox;

        debug!(mailbox = mailbox.as_str(), "mailbox found");

        let resp: MailboxResponse = client
            .service_request_tutanota(
                Method::GET,
                &format!("mailbox/{mailbox}"),
                &(),
                Some(&session.access_token),
            )
            .await
            .context("get mailbox")?;
        let folders = resp.folders.folders;

        debug!(folders = folders.as_str(), "folders found");

        let group_keys = Arc::new(session.group_keys.clone());
        let stream = client
            .stream::<FolderResponse>(
                &format!("mailfolder/{folders}"),
                Some(&session.access_token),
            )
            .and_then(move |f| {
                let group_keys = Arc::clone(&group_keys);
                async move {
                    let session_key = decrypt_key(
                        group_keys
                            .get(&f.owner_group)
                            .context("getting owner group key")?,
                        f.owner_enc_session_key.as_ref(),
                    )
                    .context("decrypting session key")?;

                    let name = if f.folder_type == MailFolderType::Custom {
                        String::from_utf8(
                            decrypt_value(&session_key, f.name.as_ref())
                                .context("decrypt folder name")?,
                        )
                        .context("invalid UTF8 string")?
                    } else {
                        f.folder_type.name().to_owned()
                    };

                    Ok(Folder {
                        name,
                        mails: f.mails,
                    })
                }
            });

        Ok(stream)
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
