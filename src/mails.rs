use std::path::Path;

use anyhow::{bail, Context, Result};
use futures::{Stream, TryStreamExt};

use crate::{client::Client, folders::Folder, proto::MailReponse, session::Session};

#[derive(Debug)]
pub struct Mail {
    pub folder_id: String,
    pub mail_id: String,
}

impl Mail {
    pub fn list(
        client: &Client,
        session: &Session,
        folder: &Folder,
    ) -> impl Stream<Item = Result<Mail>> {
        client
            .stream::<MailReponse>(
                &format!("mail/{}", folder.mails),
                Some(&session.access_token),
            )
            .and_then(move |m| async move {
                Ok(Mail {
                    folder_id: m.id[0].clone(),
                    mail_id: m.id[1].clone(),
                })
            })
    }

    pub async fn download(&self, client: &Client, target_file: &Path) -> Result<()> {
        Ok(())
    }
}
