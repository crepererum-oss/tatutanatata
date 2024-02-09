use std::collections::{HashMap, hash_map::Entry};

use anyhow::{Result, Context, bail};
use reqwest::Method;

use crate::{client::Client, login::Session, proto::{UserResponse, GroupType}};

pub async fn get_folders(client: &Client, session: &Session) -> Result<()> {
    let resp: UserResponse = client.service_requst(Method::GET, &format!("user/{}", session.user_id), &(), Some(&session.access_token)).await.context("get user")?;

    let mut memberships = HashMap::with_capacity(resp.memberships.len());
    for membership in resp.memberships {
        match memberships.entry(membership.group_type) {
            Entry::Vacant(v) => {
                v.insert(membership);
            }
            Entry::Occupied(_) => bail!("duplicate group membership for type {:?}", membership.group_type),
        }
    }

    let mail_group = memberships.get(&GroupType::Mail).context("no mail group found")?;

    Ok(())
}
