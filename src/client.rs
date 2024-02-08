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
        req: &Req,
    ) -> Result<Resp>
    where
        Req: serde::Serialize,
        Resp: DeserializeOwned,
    {
        debug!(%method, path, "service request",);

        let resp = self
            .inner
            .request(method, format!("https://app.tuta.com/rest/sys/{path}"))
            .json(req)
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
