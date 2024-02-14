use std::{path::Path, sync::Arc};

use anyhow::{bail, Context, Result};
use futures::{Stream, TryStreamExt};
use reqwest::Method;

use crate::{
    blob::get_blob,
    client::Client,
    compression::decompress_value,
    crypto::encryption::{decrypt_key, decrypt_value},
    folders::Folder,
    proto::{
        BlobAccessTokenServiceRequest, BlobAccessTokenServiceResponse, BlobReadRequest,
        MailDetailsBlob, MailReponse,
    },
    session::{GroupKeys, Session},
};

#[derive(Debug)]
pub struct Mail {
    pub folder_id: String,
    pub mail_id: String,
    pub archive_id: String,
    pub blob_id: String,
    pub session_key: Vec<u8>,
}

impl Mail {
    pub fn list(
        client: &Client,
        session: &Session,
        folder: &Folder,
    ) -> impl Stream<Item = Result<Mail>> {
        let group_keys = Arc::clone(&session.group_keys);
        client
            .stream::<MailReponse>(
                &format!("mail/{}", folder.mails),
                Some(&session.access_token),
            )
            .and_then(move |m| {
                let group_keys = Arc::clone(&group_keys);
                async move { Self::decode(m, &group_keys) }
            })
    }

    fn decode(resp: MailReponse, group_keys: &GroupKeys) -> Result<Self> {
        let session_key = decrypt_key(
            group_keys
                .get(&resp.owner_group)
                .context("getting owner group key")?,
            resp.owner_enc_session_key.as_ref(),
        )
        .context("decrypting session key")?;

        Ok(Mail {
            folder_id: resp.id[0].clone(),
            mail_id: resp.id[1].clone(),
            archive_id: resp.mail_details[0].clone(),
            blob_id: resp.mail_details[1].clone(),
            session_key,
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

        let body = decrypt_value(
            &self.session_key,
            mail_details.details.body.compressed_text.as_ref(),
        )
        .context("decrypt body")?;
        let body = decompress_value(&body).context("decompress body")?;
        let body = String::from_utf8(body).context("decode body")?;
        dbg!(body);

        Ok(())
    }
}
