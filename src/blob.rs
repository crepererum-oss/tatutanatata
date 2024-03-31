use anyhow::{bail, Context, Result};
use reqwest::Method;

use crate::{
    client::Client,
    proto::messages::{
        BlobAccessTokenServiceRequest, BlobAccessTokenServiceResponse, BlobReadRequest,
        MailDetailsBlob,
    },
    session::Session,
};

pub(crate) async fn get_mail_blob(
    client: &Client,
    session: &Session,
    archive_id: &str,
    blob_id: &str,
) -> Result<MailDetailsBlob> {
    let access = get_access(client, session, archive_id)
        .await
        .context("get blob access")?;

    let resp: Vec<MailDetailsBlob> = client
        .mail_blob_request(
            &access.server_url,
            &format!("maildetailsblob/{archive_id}"),
            &session.access_token,
            &[blob_id],
            &access.blob_access_token,
        )
        .await
        .context("blob download")?;

    if resp.len() != 1 {
        bail!("invalid reponse length")
    }

    Ok(resp.into_iter().next().expect("checked length"))
}

async fn get_access(client: &Client, session: &Session, archive_id: &str) -> Result<BlobAccess> {
    let req = BlobAccessTokenServiceRequest {
        format: Default::default(),
        archive_data_type: Default::default(),
        read: BlobReadRequest {
            id: "MR9cbw".to_owned(),
            archive_id: archive_id.to_owned(),
            instance_ids: vec![],
            instance_list_id: Default::default(),
        },
        write: Default::default(),
    };
    let resp: BlobAccessTokenServiceResponse = client
        .service_request_storage(
            Method::POST,
            "blobaccesstokenservice",
            &req,
            Some(&session.access_token),
        )
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
