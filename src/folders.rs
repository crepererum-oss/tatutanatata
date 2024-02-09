use std::collections::{hash_map::Entry, HashMap};

use anyhow::{bail, Context, Result};
use reqwest::Method;
use tracing::debug;

use crate::{
    client::Client,
    proto::{GroupType, UserMembership, UserResponse},
    session::Session,
};

pub async fn get_folders(client: &Client, session: &Session) -> Result<()> {
    let mail_group = get_mail_membership(session).context("get mail group")?;

    Ok(())
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
