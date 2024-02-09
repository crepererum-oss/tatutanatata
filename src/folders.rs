use std::collections::{hash_map::Entry, HashMap};

use anyhow::{bail, Context, Result};
use reqwest::Method;
use tracing::debug;

use crate::{
    client::Client,
    proto::{FolderResponse, GroupType, MailboxGroupRootResponse, MailboxResponse, UserMembership},
    session::Session,
};

#[derive(Debug)]
pub struct Folder {
    pub name: String,
    pub mails: String,
}

pub async fn get_folders(client: &Client, session: &Session) -> Result<Vec<Folder>> {
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

    let resp: Vec<FolderResponse> = client
        .service_request_tutanota(
            Method::GET,
            &format!("mailfolder/{folders}?start=------------&count=1000&reverse=false"),
            &(),
            Some(&session.access_token),
        )
        .await
        .context("get folders")?;

    // TODO: decrypt name for custom folders
    Ok(resp
        .into_iter()
        .map(|f| Folder {
            name: f.folder_type.name().to_owned(),
            mails: f.mails,
        })
        .collect())
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
