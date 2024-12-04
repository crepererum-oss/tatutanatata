use std::sync::Arc;

use anyhow::{bail, ensure, Context, Result};
use chrono::{DateTime, Utc};
use futures::{Stream, TryStreamExt};
use reqwest::Method;
use tracing::warn;

use crate::{
    blob::{get_attachment_blob, get_mail_blob, get_mail_draft_blob},
    client::{Client, Prefix, Request, DEFAULT_HOST},
    compression::decompress_value,
    crypto::encryption::{decrypt_key, decrypt_value},
    folders::Folder,
    proto::{
        keys::Key,
        messages::{FileReponse, MailAddress, MailReponse},
    },
    session::{GroupKeys, Session},
};

#[derive(Debug)]
pub(crate) struct Address {
    pub(crate) mail: String,
    pub(crate) name: String,
}

impl Address {
    fn decode(addr: MailAddress, session_key: Key) -> Result<Self> {
        let name = decrypt_value(session_key, &addr.name).context("decrypt name")?;
        let name = String::from_utf8(name).context("decode name string")?;

        Ok(Self {
            mail: addr.address,
            name,
        })
    }
}

#[derive(Debug)]
pub(crate) struct Mail {
    #[allow(dead_code)]
    pub(crate) folder_id: String,
    pub(crate) mail_id: String,
    pub(crate) archive_id: String,
    pub(crate) blob_id: String,
    pub(crate) is_draft: bool,
    pub(crate) session_key: Key,
    pub(crate) date: DateTime<Utc>,
    pub(crate) subject: String,
    pub(crate) sender: Address,
    pub(crate) attachments: Vec<[String; 2]>,
}

impl Mail {
    pub(crate) fn list(
        client: &Client,
        session: &Session,
        folder: &Folder,
    ) -> impl Stream<Item = Result<Arc<Self>>> {
        let group_keys = Arc::clone(&session.group_keys);
        let folder_id = folder.id.clone();
        client
            .stream::<MailReponse>(
                &format!("mail/{}", folder.mails),
                Some(&session.access_token),
            )
            .and_then(move |m| {
                let group_keys = Arc::clone(&group_keys);
                let folder_id = folder_id.clone();
                async move {
                    let mail = Self::decode(m, &group_keys, folder_id)?;
                    Ok(Arc::new(mail))
                }
            })
    }

    fn decode(resp: MailReponse, group_keys: &GroupKeys, folder_id: String) -> Result<Self> {
        let session_key = decrypt_key(
            group_keys
                .get(&resp.owner_group)
                .context("getting owner group key")?,
            resp.owner_enc_session_key,
        )
        .context("decrypting session key")?;

        let subject = decrypt_value(session_key, &resp.subject).context("decrypt subject")?;
        let subject = String::from_utf8(subject).context("decode string")?;

        let sender = Address::decode(resp.sender, session_key).context("decode sender")?;

        let ([archive_id, blob_id], is_draft) = match (resp.mail_details, resp.mail_details_draft) {
            (Some(_), Some(_)) => {
                bail!("mail as both `mailDetails` and `mailDetailsDraft`");
            }
            (Some(x), None) => (x, false),
            (None, Some(x)) => (x, true),
            (None, None) => {
                bail!("mail has neither `mailDetails` nor `mailDetailsDraft`");
            }
        };

        Ok(Self {
            folder_id,
            mail_id: resp.id[1].clone(),
            archive_id,
            blob_id,
            is_draft,
            session_key,
            date: resp.received_date.0,
            subject,
            sender,
            attachments: resp.attachments,
        })
    }

    pub(crate) fn ui_url(&self) -> String {
        format!("{}/mail/{}/{}", DEFAULT_HOST, self.folder_id, self.mail_id)
    }

    pub(crate) async fn download(
        self: Arc<Self>,
        client: &Client,
        session: &Session,
    ) -> Result<DownloadedMail> {
        let mail_details = if self.is_draft {
            get_mail_draft_blob(client, session, &self.archive_id, &self.blob_id)
                .await
                .context("download mail draft details")?
                .details
        } else {
            get_mail_blob(client, session, &self.archive_id, &self.blob_id)
                .await
                .context("download mail details")?
                .details
        };

        let body = decrypt_and_decompress(
            self.session_key,
            mail_details.body.text.as_deref(),
            mail_details.body.compressed_text.as_deref(),
        )
        .context("decode body")?;

        let headers = if let Some(headers) = mail_details.headers {
            let headers = decrypt_and_decompress(
                self.session_key,
                headers.headers.as_deref(),
                headers.compressed_headers.as_deref(),
            )
            .context("decode headers")?;
            let headers = String::from_utf8(headers).context("decode headers string")?;

            Some(headers)
        } else {
            None
        };

        let bcc = mail_details
            .recipients
            .bcc_recipients
            .into_iter()
            .map(|addr| Address::decode(addr, self.session_key))
            .collect::<Result<Vec<_>>>()
            .context("decode BCC")?;
        let cc = mail_details
            .recipients
            .cc_recipients
            .into_iter()
            .map(|addr| Address::decode(addr, self.session_key))
            .collect::<Result<Vec<_>>>()
            .context("decode CC")?;
        let to = mail_details
            .recipients
            .to_recipients
            .into_iter()
            .map(|addr| Address::decode(addr, self.session_key))
            .collect::<Result<Vec<_>>>()
            .context("decode To")?;

        let mut attachments = vec![];
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
                .do_json(Request {
                    method: Method::GET,
                    host: DEFAULT_HOST,
                    prefix: Prefix::Tutanota,
                    path: &format!("file/{group}"),
                    data: &(),
                    access_token: Some(&session.access_token),
                    query: &[("ids", &ids.join(","))],
                })
                .await
                .context("get file infos")?;

            ensure!(
                ids.len() == files.len(),
                "attachment IDs and files match, but got {} IDs and {} files",
                ids.len(),
                files.len(),
            );
            for (idx, (id, file)) in ids.into_iter().zip(files).enumerate() {
                attachments.push(
                    Self::download_file(client, session, group, file, id)
                        .await
                        .with_context(|| format!("download file #{}", idx + 1))?,
                );
            }
        }

        Ok(DownloadedMail {
            mail: self,
            headers,
            body,
            attachments,
            bcc,
            cc,
            to,
        })
    }

    async fn download_file(
        client: &Client,
        session: &Session,
        group: &str,
        file: FileReponse,
        id: &str,
    ) -> Result<Attachment> {
        let session_key = decrypt_key(
            session
                .group_keys
                .get(&file.owner_group)
                .context("getting file owner group key")?,
            file.owner_enc_session_key,
        )
        .context("decrypting file session key")?;

        let cid = if let Some(cid) = &file.cid {
            let cid = decrypt_value(session_key, cid).context("decrypt file content ID")?;
            let cid = String::from_utf8(cid).context("decode cid")?;
            Some(cid)
        } else {
            None
        };

        let mime_type = decrypt_value(session_key, file.mime_type.as_ref())
            .context("decrypt file mime type")?;
        let mime_type = String::from_utf8(mime_type).context("decode mime_type")?;

        let name = decrypt_value(session_key, file.name.as_ref()).context("decrypt file name")?;
        let name = String::from_utf8(name).context("decode name")?;

        let mut data_all = Vec::with_capacity(file.size.0 as usize);
        let mut encrypted_size_sum = 0;
        for blob in file.blobs {
            let data =
                get_attachment_blob(client, session, &blob.archive_id, &blob.blob_id, group, id)
                    .await
                    .context("download attachment")?;
            ensure!(
                data.len() == blob.size.0 as usize,
                "encrypted blob data size is wrong, should be {} bytes but got {} bytes",
                blob.size.0,
                data.len(),
            );
            encrypted_size_sum += data.len();
            let mut data = decrypt_value(session_key, &data).context("decrypt attachment data")?;
            data_all.append(&mut data);
        }
        if encrypted_size_sum != file.size.0 as usize {
            warn!(
                actual=encrypted_size_sum,
                expected=file.size.0,
                related_issue="https://github.com/crepererum-oss/tatutanatata/issues/278",
                "encrypted blobs do not add up to file size, this seems to happen for some older data",
            );
        }

        Ok(Attachment {
            cid,
            mime_type,
            name,
            data: data_all,
        })
    }
}

fn decrypt_and_decompress(
    encryption_key: Key,
    plain: Option<&[u8]>,
    compressed: Option<&[u8]>,
) -> Result<Vec<u8>> {
    match (plain, compressed) {
        (Some(data), _) => {
            let data = decrypt_value(encryption_key, data).context("decrypt")?;
            Ok(data)
        }
        (None, Some(data)) => {
            let data = decrypt_value(encryption_key, data).context("decrypt")?;
            let data = decompress_value(&data).context("decompress")?;
            Ok(data)
        }
        (None, None) => {
            bail!("neither compressed or uncompressed data available")
        }
    }
}

#[derive(Debug)]
pub(crate) struct DownloadedMail {
    pub(crate) mail: Arc<Mail>,
    pub(crate) headers: Option<String>,
    pub(crate) body: Vec<u8>,
    pub(crate) attachments: Vec<Attachment>,
    pub(crate) bcc: Vec<Address>,
    pub(crate) cc: Vec<Address>,
    pub(crate) to: Vec<Address>,
}

#[derive(Debug)]
pub(crate) struct Attachment {
    pub(crate) cid: Option<String>,
    pub(crate) mime_type: String,
    pub(crate) name: String,
    pub(crate) data: Vec<u8>,
}
