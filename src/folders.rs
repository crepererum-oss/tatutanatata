use std::collections::{hash_map::Entry, HashMap};

use anyhow::{bail, Context, Result};
use reqwest::Method;

use crate::{
    client::Client,
    proto::{GroupType, UserResponse},
    session::Session,
};

pub async fn get_folders(client: &Client, session: &Session) -> Result<()> {
    let resp: UserResponse = client
        .service_requst(
            Method::GET,
            &format!("user/{}", session.user_id),
            &(),
            Some(&session.access_token),
        )
        .await
        .context("get user")?;

    let mut memberships = HashMap::with_capacity(resp.memberships.len());
    for membership in resp.memberships {
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

    let mail_group = memberships
        .get(&GroupType::Mail)
        .context("no mail group found")?;

    Ok(())
}
