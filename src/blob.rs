use anyhow::{bail, Context, Result};
use reqwest::Method;
use serde::de::DeserializeOwned;

use crate::{
    client::Client,
    proto::{BlobAccessTokenServiceRequest, BlobAccessTokenServiceResponse, BlobReadRequest},
    session::Session,
};

pub(crate) async fn get_blob<Resp>(
    client: &Client,
    session: &Session,
    archive_id: &str,
    blob_id: &str,
) -> Result<Resp>
where
    Resp: DeserializeOwned,
{
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

    let resp: Vec<Resp> = client
        .blob_request(
            &server.url,
            &format!("maildetailsblob/{archive_id}"),
            &session.access_token,
            &[blob_id],
            &resp.blob_access_info.blob_access_token,
        )
        .await
        .context("blob download")?;

    if resp.len() != 1 {
        bail!("invalid reponse length")
    }

    Ok(resp.into_iter().next().expect("checked length"))
}
