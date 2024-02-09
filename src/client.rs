use anyhow::{Context, Result};
use reqwest::{Method, Response};
use serde::de::DeserializeOwned;
use tracing::debug;

use crate::proto::Base64Url;

#[derive(Debug)]
pub struct Client {
    inner: reqwest::Client,
}

impl Client {
    pub fn try_new() -> Result<Self> {
        let inner = reqwest::Client::builder()
            .build()
            .context("set up HTTPs client")?;
        Ok(Self { inner })
    }

    pub async fn service_requst<Req, Resp>(
        &self,
        method: Method,
        path: &str,
        data: &Req,
        access_token: Option<&Base64Url>,
    ) -> Result<Resp>
    where
        Req: serde::Serialize,
        Resp: DeserializeOwned,
    {
        let resp = self
            .service_requst_no_response(method, path, data, access_token)
            .await?
            .json::<Resp>()
            .await
            .context("fetch JSON response")?;

        Ok(resp)
    }

    pub async fn service_requst_no_response<Req>(
        &self,
        method: Method,
        path: &str,
        data: &Req,
        access_token: Option<&Base64Url>,
    ) -> Result<Response>
    where
        Req: serde::Serialize,
    {
        debug!(%method, path, "service request",);

        let mut req = self
            .inner
            .request(method, format!("https://app.tuta.com/rest/sys/{path}"));

        if let Some(access_token) = access_token {
            req = req.header("accessToken", access_token.to_string());
        }

        let resp = req
            .json(data)
            .send()
            .await
            .context("initial request")?
            .error_for_status()
            .context("return status")?;

        Ok(resp)
    }
}
