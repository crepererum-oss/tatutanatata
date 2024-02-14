use std::{collections::VecDeque, sync::Arc};

use anyhow::{Context, Result};
use futures::Stream;
use reqwest::{Method, Response};
use serde::de::DeserializeOwned;
use tracing::debug;

use crate::{
    constants::APP_USER_AGENT,
    proto::{Base64Url, Entity},
};

const STREAM_BATCH_SIZE: u64 = 1000;
const DEFAULT_HOST: &str = "https://app.tuta.com";

#[derive(Debug, Clone)]
pub struct Client {
    inner: reqwest::Client,
}

impl Client {
    pub fn try_new() -> Result<Self> {
        let inner = reqwest::Client::builder()
            .min_tls_version(reqwest::tls::Version::TLS_1_3)
            .http2_prior_knowledge()
            .user_agent(APP_USER_AGENT)
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
        self.do_json(method, DEFAULT_HOST, "sys", path, data, access_token, &[])
            .await
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
        self.do_json(
            method,
            DEFAULT_HOST,
            "tutanota",
            path,
            data,
            access_token,
            &[],
        )
        .await
    }

    pub async fn service_request_storage<Req, Resp>(
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
        self.do_json(
            method,
            DEFAULT_HOST,
            "storage",
            path,
            data,
            access_token,
            &[],
        )
        .await
    }

    pub async fn blob_request<Resp>(
        &self,
        host: &str,
        path: &str,
        access_token: &Base64Url,
        ids: &[&str],
        blob_access_token: &str,
    ) -> Result<Resp>
    where
        Resp: DeserializeOwned,
    {
        self.do_json(
            Method::GET,
            host,
            "tutanota",
            path,
            &(),
            None,
            &[
                ("accessToken", &access_token.to_string()),
                ("ids", &ids.join(",")),
                ("blobAccessToken", blob_access_token),
            ],
        )
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
        self.do_request(method, DEFAULT_HOST, "sys", path, data, access_token, &[])
            .await
    }

    pub fn stream<Resp>(
        &self,
        path: &str,
        access_token: Option<&Base64Url>,
    ) -> impl Stream<Item = Result<Resp>>
    where
        Resp: DeserializeOwned + Entity,
    {
        let state = StreamState {
            buffer: VecDeque::default(),
            next_start: "------------".to_owned(),
        };
        let path = Arc::new(path.to_owned());
        let access_token = Arc::new(access_token.cloned());
        let this = self.clone();

        futures::stream::try_unfold(state, move |mut state: StreamState<Resp>| {
            let path = Arc::clone(&path);
            let access_token = Arc::clone(&access_token);
            let this = this.clone();
            async move {
                loop {
                    if let Some(next) = state.buffer.pop_front() {
                        return Ok(Some((next, state)));
                    }

                    // buffer empty
                    debug!(
                        path = path.as_str(),
                        start = state.next_start.as_str(),
                        "fetch new page",
                    );
                    state.buffer = this
                        .do_json::<(), Vec<Resp>>(
                            Method::GET,
                            DEFAULT_HOST,
                            "tutanota",
                            &path,
                            &(),
                            access_token.as_ref().as_ref(),
                            &[
                                ("start", &state.next_start),
                                ("count", &STREAM_BATCH_SIZE.to_string()),
                                ("reverse", "false"),
                            ],
                        )
                        .await
                        .context("fetch next page")?
                        .into();
                    match state.buffer.back() {
                        None => {
                            // reached end
                            return Ok(None);
                        }
                        Some(o) => {
                            state.next_start = o.id().to_owned();
                        }
                    }
                }
            }
        })
    }

    async fn do_json<Req, Resp>(
        &self,
        method: Method,
        host: &str,
        prefix: &str,
        path: &str,
        data: &Req,
        access_token: Option<&Base64Url>,
        query: &[(&str, &str)],
    ) -> Result<Resp>
    where
        Req: serde::Serialize,
        Resp: DeserializeOwned,
    {
        let resp = self
            .do_request(method, host, prefix, path, data, access_token, query)
            .await?
            .json::<Resp>()
            .await
            .context("fetch JSON response")?;

        Ok(resp)
    }

    async fn do_request<Req>(
        &self,
        method: Method,
        host: &str,
        prefix: &str,
        path: &str,
        data: &Req,
        access_token: Option<&Base64Url>,
        query: &[(&str, &str)],
    ) -> Result<Response>
    where
        Req: serde::Serialize,
    {
        debug!(%method, prefix, path, "service request",);

        let mut req = self
            .inner
            .request(method, format!("{host}/rest/{prefix}/{path}"));

        if let Some(access_token) = access_token {
            req = req.header("accessToken", access_token.to_string());
        }

        let resp = req
            .json(data)
            .query(query)
            .send()
            .await
            .context("initial request")?
            .error_for_status()
            .context("return status")?;

        Ok(resp)
    }
}

struct StreamState<T> {
    buffer: VecDeque<T>,
    next_start: String,
}
