use anyhow::{Context, Result};
use reqwest::Method;
use serde::de::DeserializeOwned;
use tracing::debug;

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
        access_token: Option<&str>,
    ) -> Result<Resp>
    where
        Req: serde::Serialize,
        Resp: DeserializeOwned,
    {
        debug!(%method, path, "service request",);

        let mut req = self
            .inner
            .request(method, format!("https://app.tuta.com/rest/sys/{path}"));

        if let Some(access_token) = access_token {
            req = req.header("accessToken", access_token);
        }

        let resp = req.json(data)
            .send()
            .await
            .context("initial request")?
            .error_for_status()
            .context("return status")?
            .json::<Resp>()
            .await
            .context("fetch JSON response")?;

        Ok(resp)
    }
}
