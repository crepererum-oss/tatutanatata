use std::sync::OnceLock;

use anyhow::{Context, Result};
use itertools::Itertools;

use crate::{
    mails::{DownloadedMail, Mail},
    proto::binary::Base64String,
};

static LINE_ENDING_RE: OnceLock<regex::Regex> = OnceLock::new();
static BOUNDARY_RE: OnceLock<regex::Regex> = OnceLock::new();
const NEWLINE: &str = "\r\n";

pub(crate) fn emit_eml(mail: &DownloadedMail) -> Result<String> {
    let mut lines = Vec::new();

    // headers
    let boundary = if let Some(headers) = &mail.headers {
        let mut headers = split_header_lines(headers);
        let boundary = get_boundary(&headers).context("get boundary")?;

        lines.append(&mut headers);
        boundary
    } else {
        let boundary = "----------79Bu5A16qPEYcVIZL@tutanota".to_owned();
        synthesize_headers(&mail.mail, &boundary, &mut lines);
        boundary
    };

    // body
    write_intermediate_delimiter(&mut lines, &boundary);
    let body = Base64String::from(mail.body.clone());
    lines.push("Content-Type: text/html; charset=UTF-8".to_owned());
    lines.push("Content-Transfer-Encoding: base64".to_owned());
    lines.push("".to_owned());
    write_chunked(&mut lines, &body.to_string());

    // attachments
    for attachment in &mail.attachments {
        write_intermediate_delimiter(&mut lines, &boundary);
        lines.push(format!(
            "Content-Type: {}; name={}",
            attachment.mime_type,
            utf8_header_value(&attachment.name)
        ));
        lines.push("Content-Transfer-Encoding: base64".to_owned());
        lines.push(format!(
            "Content-Disposition: attachment; filename={}",
            utf8_header_value(&attachment.name)
        ));
        lines.push(format!("Content-Id: <{}>", attachment.cid));
        lines.push("".to_owned());
        write_chunked(
            &mut lines,
            &Base64String::from(attachment.data.clone()).to_string(),
        );
    }

    write_final_delimiter(&mut lines, &boundary);
    Ok(lines.join(NEWLINE))
}

/// Create headers from metadata and return multipart boundary.
fn synthesize_headers(mail: &Mail, boundary: &str, lines: &mut Vec<String>) {
    lines.push(format!("From: {} <{}>", mail.sender_name, mail.sender_mail));

    lines.push("MIME-Version: 1.0".to_owned());

    if mail.subject.is_empty() {
        lines.push("Subject: ".to_owned());
    } else {
        lines.push(format!("Subject: {}", utf8_header_value(&mail.subject),));
    };

    lines.push(format!(
        "Content-Type: multipart/related; boundary=\"{}\"",
        boundary
    ));
}

fn line_ending_re() -> &'static regex::Regex {
    LINE_ENDING_RE.get_or_init(|| regex::Regex::new(r#"\r?\n"#).expect("valid regex"))
}

fn boundary_re() -> &'static regex::Regex {
    BOUNDARY_RE.get_or_init(|| {
        regex::Regex::new(r#"Content-Type: .*boundary="(?<boundary>[^"]*)""#).expect("valid regex")
    })
}

/// Upstream provides `\n` line endings for headers but we need `\r\n`
fn split_header_lines(headers: &str) -> Vec<String> {
    line_ending_re()
        .split(headers)
        .map(|s| s.to_owned())
        .collect()
}

/// Extract boundary from headers
fn get_boundary(headers: &[String]) -> Result<String> {
    let boundary_re = boundary_re();

    headers
        .iter()
        .find_map(|line| boundary_re.captures(line).and_then(|c| c.name("boundary")))
        .map(|s| s.as_str().to_owned())
        .context("boundary not found")
}

/// See <https://www.w3.org/Protocols/rfc1341/7_2_Multipart.html>.
fn write_intermediate_delimiter(lines: &mut Vec<String>, boundary: &str) {
    lines.push("".to_owned());
    lines.push(format!("--{}", boundary));
}

/// See <https://www.w3.org/Protocols/rfc1341/7_2_Multipart.html>.
fn write_final_delimiter(lines: &mut Vec<String>, boundary: &str) {
    lines.push("".to_owned());
    lines.push(format!("--{}--", boundary));
}

fn write_chunked(lines: &mut Vec<String>, s: &str) {
    for chunk in &s.chars().chunks(78) {
        lines.push(chunk.collect());
    }
}

fn utf8_header_value(s: &str) -> String {
    format!("=?UTF-8?B?{}?=", Base64String::from(s.as_bytes()))
}

#[cfg(test)]
mod tests {
    use chrono::DateTime;

    use crate::{mails::Attachment, proto::keys::Key};

    use super::*;

    #[test]
    fn test_simple() {
        let eml = emit_eml(&DownloadedMail {
            mail: Mail {
                folder_id: "folder_id".to_owned(),
                mail_id: "mail_id".to_owned(),
                archive_id: "archive_id".to_owned(),
                blob_id: "blob_id".to_owned(),
                session_key: Key::Aes256([0; 32]),
                date: DateTime::parse_from_rfc3339("2020-03-04T11:22:33Z")
                    .unwrap()
                    .to_utc(),
                subject: "Hällö".to_owned(),
                sender_mail: "foo@example.com".to_owned(),
                sender_name: "Me".to_owned(),
                attachments: vec![],
            },
            headers: Some(
                "From: foo@example.com\nContent-Type: multipart/related; boundary=\"myboundary\""
                    .to_owned(),
            ),
            body: b"hello world".to_vec(),
            attachments: vec![],
        })
        .unwrap();
        insta::assert_snapshot!(eml, @r###"
        From: foo@example.com
        Content-Type: multipart/related; boundary="myboundary"

        --myboundary
        Content-Type: text/html; charset=UTF-8
        Content-Transfer-Encoding: base64

        aGVsbG8gd29ybGQ=

        --myboundary--
        "###);
    }

    #[test]
    fn test_attachments() {
        let eml = emit_eml(&DownloadedMail {
            mail: Mail {
                folder_id: "folder_id".to_owned(),
                mail_id: "mail_id".to_owned(),
                archive_id: "archive_id".to_owned(),
                blob_id: "blob_id".to_owned(),
                session_key: Key::Aes256([0; 32]),
                date: DateTime::parse_from_rfc3339("2020-03-04T11:22:33Z")
                    .unwrap()
                    .to_utc(),
                subject: "Hällö".to_owned(),
                sender_mail: "foo@example.com".to_owned(),
                sender_name: "Me".to_owned(),
                attachments: vec![
                    ["a".to_owned(), "b".to_owned()],
                    ["c".to_owned(), "d".to_owned()],
                ],
            },
            headers: Some(
                "From: foo@example.com\nContent-Type: multipart/related; boundary=\"myboundary\""
                    .to_owned(),
            ),
            body: b"hello world".to_vec(),
            attachments: vec![
                Attachment {
                    cid: "cid001".to_owned(),
                    mime_type: "image/jpeg".to_owned(),
                    name: "föo.jpg".to_owned(),
                    data: b"foobar".to_vec(),
                },
                Attachment {
                    cid: "cid002".to_owned(),
                    mime_type: "image/new".to_owned(),
                    name: "å".to_owned(),
                    data: b"x".to_vec(),
                },
            ],
        })
        .unwrap();
        insta::assert_snapshot!(eml, @r###"
        From: foo@example.com
        Content-Type: multipart/related; boundary="myboundary"

        --myboundary
        Content-Type: text/html; charset=UTF-8
        Content-Transfer-Encoding: base64

        aGVsbG8gd29ybGQ=

        --myboundary
        Content-Type: image/jpeg; name==?UTF-8?B?ZsO2by5qcGc=?=
        Content-Transfer-Encoding: base64
        Content-Disposition: attachment; filename==?UTF-8?B?ZsO2by5qcGc=?=
        Content-Id: <cid001>

        Zm9vYmFy

        --myboundary
        Content-Type: image/new; name==?UTF-8?B?w6U=?=
        Content-Transfer-Encoding: base64
        Content-Disposition: attachment; filename==?UTF-8?B?w6U=?=
        Content-Id: <cid002>

        eA==

        --myboundary--
        "###);
    }

    #[test]
    fn test_synthesize_headers() {
        let eml = emit_eml(&DownloadedMail {
            mail: Mail {
                folder_id: "folder_id".to_owned(),
                mail_id: "mail_id".to_owned(),
                archive_id: "archive_id".to_owned(),
                blob_id: "blob_id".to_owned(),
                session_key: Key::Aes256([0; 32]),
                date: DateTime::parse_from_rfc3339("2020-03-04T11:22:33Z")
                    .unwrap()
                    .to_utc(),
                subject: "Hällö".to_owned(),
                sender_mail: "foo@example.com".to_owned(),
                sender_name: "Me".to_owned(),
                attachments: vec![],
            },
            headers: None,
            body: b"hello world".to_vec(),
            attachments: vec![],
        })
        .unwrap();
        insta::assert_snapshot!(eml, @r###"
        From: Me <foo@example.com>
        MIME-Version: 1.0
        Subject: =?UTF-8?B?SMOkbGzDtg==?=
        Content-Type: multipart/related; boundary="----------79Bu5A16qPEYcVIZL@tutanota"

        ------------79Bu5A16qPEYcVIZL@tutanota
        Content-Type: text/html; charset=UTF-8
        Content-Transfer-Encoding: base64

        aGVsbG8gd29ybGQ=

        ------------79Bu5A16qPEYcVIZL@tutanota--
        "###);
    }
}
