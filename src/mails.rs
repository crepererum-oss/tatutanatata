use std::path::Path;

use anyhow::{bail, Context, Result};
use futures::{Stream, TryStreamExt};
use reqwest::Method;

use crate::{
    blob::get_blob,
    client::Client,
    folders::Folder,
    proto::{
        BlobAccessTokenServiceRequest, BlobAccessTokenServiceResponse, BlobReadRequest,
        MailDetailsBlob, MailReponse,
    },
    session::Session,
};

#[derive(Debug)]
pub struct Mail {
    pub folder_id: String,
    pub mail_id: String,
    pub archive_id: String,
    pub blob_id: String,
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
                    archive_id: m.mail_details[0].clone(),
                    blob_id: m.mail_details[1].clone(),
                })
            })
    }

    pub async fn download(
        &self,
        client: &Client,
        session: &Session,
        target_file: &Path,
    ) -> Result<()> {
        let mail_details: MailDetailsBlob =
            get_blob(client, session, &self.archive_id, &self.blob_id)
                .await
                .context("download mail details")?;

        Ok(())
    }
}
