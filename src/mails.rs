use std::{
    path::Path,
    sync::{Arc, OnceLock},
};

use anyhow::{Context, Result};
use chrono::NaiveDateTime;
use futures::{Stream, TryStreamExt};
use itertools::Itertools;

use crate::{
    blob::get_blob,
    client::Client,
    compression::decompress_value,
    crypto::encryption::{decrypt_key, decrypt_value},
    file_output::write_to_file,
    folders::Folder,
    proto::{
        binary::Base64String,
        keys::Key,
        messages::{MailDetailsBlob, MailReponse},
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
        &self,
        client: &Client,
        session: &Session,
        target_file: &Path,
    ) -> Result<()> {
        let mail_details: MailDetailsBlob =
            get_blob(client, session, &self.archive_id, &self.blob_id)
                .await
                .context("download mail details")?;

        self.emit_eml(mail_details, target_file)
            .await
            .context("emit EML")
    }

    async fn emit_eml(&self, mail_details: MailDetailsBlob, target_file: &Path) -> Result<()> {
        let mut out = Vec::new();

        let boundary = if let Some(headers) = mail_details.details.headers {
            let headers = decrypt_value(self.session_key, headers.compressed_headers.as_ref())
                .context("decrypt headers")?;
            let headers = decompress_value(&headers).context("decompress headers")?;
            let mut headers = fix_header_line_endings(&headers);
            let boundary = get_boundary(&headers).context("get boundary")?;

            out.append(&mut headers);
            boundary
        } else {
            self.synthesize_headers(&mut out)
        };

        write_intermediate_delimiter(&mut out, &boundary);

        let body = decrypt_value(
            self.session_key,
            mail_details.details.body.compressed_text.as_ref(),
        )
        .context("decrypt body")?;
        let body = decompress_value(&body).context("decompress body")?;
        let body = Base64String::from(body);
        out.extend(b"Content-Type: text/html; charset=UTF-8");
        out.extend(NEWLINE);
        out.extend(b"Content-Transfer-Encoding: base64");
        out.extend(NEWLINE);
        out.extend(NEWLINE);
        write_chunked(&mut out, body.to_string().as_bytes());

        write_final_delimiter(&mut out, &boundary);

        write_to_file(&out, target_file)
            .await
            .context("write to output file")?;

        Ok(())
    }

    /// Create headers from metadata and return multipart boundary.
    fn synthesize_headers(&self, out: &mut Vec<u8>) -> Vec<u8> {
        let boundary = b"----------79Bu5A16qPEYcVIZL@tutanota".to_vec();

        out.extend(b"From: ");
        out.extend(self.sender_name.as_bytes());
        out.extend(b" <");
        out.extend(self.sender_mail.as_bytes());
        out.extend(b">");
        out.extend(NEWLINE);

        out.extend(b"MIME-Version: 1.0");
        out.extend(NEWLINE);

        if self.subject.is_empty() {
            out.extend(b"Subject: ");
            out.extend(NEWLINE);
        } else {
            out.extend(b"Subject: =?UTF-8?B?");
            out.extend(
                Base64String::from(self.subject.as_bytes())
                    .to_string()
                    .as_bytes(),
            );
            out.extend(b"?=");
        };

        out.extend(b"Content-Type: multipart/related; boundary=\"");
        out.extend(&boundary);
        out.extend(b"\"");
        out.extend(NEWLINE);

        boundary
    }
}

fn line_ending_re() -> &'static regex::bytes::Regex {
    LINE_ENDING_RE.get_or_init(|| regex::bytes::Regex::new(r#"\r?\n"#).expect("valid regex"))
}

fn boundary_re() -> &'static regex::bytes::Regex {
    BOUNDARY_RE.get_or_init(|| {
        regex::bytes::Regex::new(r#"Content-Type: .*boundary="(?<boundary>[^"]*)""#)
            .expect("valid regex")
    })
}

/// Upstream provides `\n` line endings for headers but we need `\r\n`
#[allow(unstable_name_collisions)]
fn fix_header_line_endings(headers: &[u8]) -> Vec<u8> {
    line_ending_re()
        .split(headers)
        .map(|s| s.to_vec())
        .intersperse(b"\r\n".to_vec())
        .concat()
}

/// Extract boundery from headers
fn get_boundary(headers: &[u8]) -> Result<Vec<u8>> {
    let boundary_re = boundary_re();

    line_ending_re()
        .split(headers)
        .find_map(|line| boundary_re.captures(line).and_then(|c| c.name("boundary")))
        .map(|s| s.as_bytes().to_owned())
        .context("boundary not found")
}

/// See <https://www.w3.org/Protocols/rfc1341/7_2_Multipart.html>.
fn write_delimiter(out: &mut Vec<u8>, boundary: &[u8]) {
    out.extend(NEWLINE);
    out.extend(NEWLINE);
    out.extend(b"--");
    out.extend(boundary);
}

fn write_intermediate_delimiter(out: &mut Vec<u8>, boundary: &[u8]) {
    write_delimiter(out, boundary);
    out.extend(NEWLINE);
}

/// See <https://www.w3.org/Protocols/rfc1341/7_2_Multipart.html>.
fn write_final_delimiter(out: &mut Vec<u8>, boundary: &[u8]) {
    write_delimiter(out, boundary);
    out.extend(b"--");
}

fn write_chunked(out: &mut Vec<u8>, s: &[u8]) {
    let mut first = false;
    for chunk in s.chunks(78) {
        if !first {
            out.extend(NEWLINE);
        }
        out.extend(chunk);
        first = false;
    }
}
