use anyhow::{bail, Context, Result};
use reqwest::Method;

use crate::{
    client::{Client, Prefix, Request, DEFAULT_HOST},
    proto::{
        enums::ArchiveDataType,
        messages::{
            BlobAccessTokenServiceRequest, BlobAccessTokenServiceResponse, BlobReadRequest,
            BlobReadRequestInstanceId, BlobServiceRequest, MailDetailsBlob,
        },
    },
    session::Session,
};

pub(crate) async fn get_mail_blob(
    client: &Client,
    session: &Session,
    archive_id: &str,
    blob_id: &str,
) -> Result<MailDetailsBlob> {
    let access = get_access(
        client,
        session,
        archive_id,
        ArchiveDataType::MailDetails,
        None,
    )
    .await
    .context("get blob access")?;

    let resp: Vec<MailDetailsBlob> = client
        .do_json(Request {
            method: Method::GET,
            host: &access.server_url,
            prefix: Prefix::Tutanota,
            path: &format!("maildetailsblob/{archive_id}"),
            data: &(),
            access_token: None,
            query: &[
                ("accessToken", &session.access_token.to_string()),
                ("ids", &[blob_id].join(",")),
                ("blobAccessToken", &access.blob_access_token),
            ],
        })
        .await
        .context("blob download")?;

    if resp.len() != 1 {
        bail!("invalid reponse length")
    }

    Ok(resp.into_iter().next().expect("checked length"))
}

pub(crate) async fn get_attachment_blob(
    client: &Client,
    session: &Session,
    archive_id: &str,
    blob_id: &str,
    instance_list_id: &str,
    instance_id: &str,
) -> Result<Vec<u8>> {
    let access = get_access(
        client,
        session,
        archive_id,
        ArchiveDataType::Attachments,
        Some((instance_list_id, instance_id)),
    )
    .await
    .context("get blob access")?;

    let data = client
        .do_bytes(Request {
            method: Method::GET,
            host: &access.server_url,
            prefix: Prefix::Storage,
            path: "blobservice",
            data: &(),
            access_token: None,
            query: &[
                ("accessToken", &session.access_token.to_string()),
                ("blobAccessToken", &access.blob_access_token),
                (
                    "_body",
                    &serde_json::to_string(&BlobServiceRequest {
                        format: Default::default(),
                        archive_id: archive_id.to_owned(),
                        blob_id: blob_id.to_owned(),
                        blob_ids: vec![],
                    })
                    .expect("serde should always work"),
                ),
            ],
        })
        .await
        .context("blob download")?;

    Ok(data.to_vec())
}

async fn get_access(
    client: &Client,
    session: &Session,
    archive_id: &str,
    archive_data_type: ArchiveDataType,
    instance: Option<(&str, &str)>,
) -> Result<BlobAccess> {
    let req = BlobAccessTokenServiceRequest {
        format: Default::default(),
        archive_data_type,
        read: BlobReadRequest {
            id: "MR9cbw".to_owned(),
            archive_id: archive_id.to_owned(),
            instance_ids: instance
                .iter()
                .map(|(_l, i)| BlobReadRequestInstanceId {
                    id: "MR9cbw".to_owned(),
                    instance_id: (*i).to_owned(),
                })
                .collect(),
            instance_list_id: instance.as_ref().map(|(l, _i)| (*l).to_owned()),
        },
        write: Default::default(),
    };
    let resp: BlobAccessTokenServiceResponse = client
        .do_json(Request {
            method: Method::POST,
            host: DEFAULT_HOST,
            prefix: Prefix::Storage,
            path: "blobaccesstokenservice",
            data: &req,
            access_token: Some(&session.access_token),
            query: &[],
        })
        .await
        .context("blob service access request")?;

    let Some(server) = resp.blob_access_info.servers.first() else {
        bail!("no blob servers provided")
    };

    Ok(BlobAccess {
        server_url: server.url.clone(),
        blob_access_token: resp.blob_access_info.blob_access_token,
    })
}

#[derive(Debug)]
pub(crate) struct BlobAccess {
    pub(crate) server_url: String,
    pub(crate) blob_access_token: String,
}
