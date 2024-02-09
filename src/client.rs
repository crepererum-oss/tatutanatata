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

    pub async fn service_request<Req, Resp>(
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
        self.do_json(method, "sys", path, data, access_token).await
    }

    pub async fn service_request_tutanota<Req, Resp>(
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
        self.do_json(method, "tutanota", path, data, access_token)
            .await
    }

    pub async fn service_request_no_response<Req>(
        &self,
        method: Method,
        path: &str,
        data: &Req,
        access_token: Option<&Base64Url>,
    ) -> Result<Response>
    where
        Req: serde::Serialize,
    {
        self.do_request(method, "sys", path, data, access_token)
            .await
    }

    async fn do_json<Req, Resp>(
        &self,
        method: Method,
        prefix: &str,
        path: &str,
        data: &Req,
        access_token: Option<&Base64Url>,
    ) -> Result<Resp>
    where
        Req: serde::Serialize,
        Resp: DeserializeOwned,
    {
        let resp = self
            .do_request(method, prefix, path, data, access_token)
            .await?
            .json::<Resp>()
            .await
            .context("fetch JSON response")?;

        Ok(resp)
    }

    async fn do_request<Req>(
        &self,
        method: Method,
        prefix: &str,
        path: &str,
        data: &Req,
        access_token: Option<&Base64Url>,
    ) -> Result<Response>
    where
        Req: serde::Serialize,
    {
        debug!(%method, prefix, path, "service request",);

        let mut req = self
            .inner
            .request(method, format!("https://app.tuta.com/rest/{prefix}/{path}"));

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
