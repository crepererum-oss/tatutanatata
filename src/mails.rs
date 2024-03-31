use std::sync::{Arc, OnceLock};

use anyhow::{bail, Context, Result};
use chrono::NaiveDateTime;
use futures::{Stream, TryStreamExt};

use crate::{
    blob::get_mail_blob,
    client::Client,
    compression::decompress_value,
    crypto::encryption::{decrypt_key, decrypt_value},
    folders::Folder,
    proto::{
        keys::Key,
        messages::{FileReponse, MailReponse},
    },
    session::{GroupKeys, Session},
};

static LINE_ENDING_RE: OnceLock<regex::bytes::Regex> = OnceLock::new();
static BOUNDARY_RE: OnceLock<regex::bytes::Regex> = OnceLock::new();
const NEWLINE: &[u8] = b"\r\n";

#[derive(Debug)]
pub(crate) struct Mail {
    #[allow(dead_code)]
    pub(crate) folder_id: String,
    pub(crate) mail_id: String,
    pub(crate) archive_id: String,
    pub(crate) blob_id: String,
    pub(crate) session_key: Key,
    pub(crate) date: NaiveDateTime,
    pub(crate) subject: String,
    pub(crate) sender_mail: String,
    pub(crate) sender_name: String,
    pub(crate) attachments: Vec<[String; 2]>,
}

impl Mail {
    pub(crate) fn list(
        client: &Client,
        session: &Session,
        folder: &Folder,
    ) -> impl Stream<Item = Result<Self>> {
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
            resp.owner_enc_session_key,
        )
        .context("decrypting session key")?;

        let subject = decrypt_value(session_key, &resp.subject).context("decrypt subject")?;
        let subject = String::from_utf8(subject).context("decode string")?;

        let sender_name =
            decrypt_value(session_key, &resp.sender.name).context("decrypt subject")?;
        let sender_name = String::from_utf8(sender_name).context("decode sender name")?;

        Ok(Self {
            folder_id: resp.id[0].clone(),
            mail_id: resp.id[1].clone(),
            archive_id: resp.mail_details[0].clone(),
            blob_id: resp.mail_details[1].clone(),
            session_key,
            date: resp.received_date.0,
            subject,
            sender_mail: resp.sender.address,
            sender_name,
            attachments: resp.attachments,
        })
    }

    pub(crate) async fn download(
        self,
        client: &Client,
        session: &Session,
    ) -> Result<DownloadedMail> {
        let mail_details = get_mail_blob(client, session, &self.archive_id, &self.blob_id)
            .await
            .context("download mail details")?;
        let body = decrypt_value(
            self.session_key,
            mail_details.details.body.compressed_text.as_ref(),
        )
        .context("decrypt body")?;
        let body = decompress_value(&body).context("decompress body")?;

        let headers = if let Some(headers) = mail_details.details.headers {
            let headers = decrypt_value(self.session_key, headers.compressed_headers.as_ref())
                .context("decrypt headers")?;
            let headers = decompress_value(&headers).context("decompress headers")?;
            let headers = String::from_utf8(headers).context("decode headers")?;

            Some(headers)
        } else {
            None
        };

        let mut attachements = vec![];
        if !self.attachments.is_empty() {
            let group = &self.attachments[0][0];
            if self.attachments.iter().any(|[g_id, _id]| g_id != group) {
                bail!("inconsistent attachement group IDs")
            }
            let ids = self
                .attachments
                .iter()
                .map(|[_g_id, id]| id.as_str())
                .collect::<Vec<_>>();
            let files: Vec<FileReponse> = client
                .file_request(&session.access_token, group, &ids)
                .await
                .context("get file infos")?;

            for file in files {
                let session_key = decrypt_key(
                    session
                        .group_keys
                        .get(&file.owner_group)
                        .context("getting file owner group key")?,
                    file.owner_enc_session_key,
                )
                .context("decrypting file session key")?;

                let cid = decrypt_value(session_key, file.cid.as_ref())
                    .context("decrypt file content ID")?;
                let cid = String::from_utf8(cid).context("decode cid")?;

                let mime_type = decrypt_value(session_key, file.mime_type.as_ref())
                    .context("decrypt file mime type")?;
                let mime_type = String::from_utf8(mime_type).context("decode mime_type")?;

                let name =
                    decrypt_value(session_key, file.name.as_ref()).context("decrypt file name")?;
                let name = String::from_utf8(name).context("decode name")?;

                attachements.push(Attachment {
                    cid,
                    mime_type,
                    name,
                });
            }
        }

        Ok(DownloadedMail {
            mail: self,
            headers,
            body,
            attachements,
        })
    }
}

#[derive(Debug)]
pub(crate) struct DownloadedMail {
    pub(crate) mail: Mail,
    pub(crate) headers: Option<String>,
    pub(crate) body: Vec<u8>,
    pub(crate) attachements: Vec<Attachment>,
}

#[derive(Debug)]
pub(crate) struct Attachment {
    pub(crate) cid: String,
    pub(crate) mime_type: String,
    pub(crate) name: String,
}
