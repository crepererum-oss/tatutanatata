use std::{collections::VecDeque, sync::Arc};

use anyhow::{Context, Result};
use futures::Stream;
use reqwest::{Method, Response};
use serde::de::DeserializeOwned;
use tracing::debug;

use crate::{
    constants::APP_USER_AGENT,
    proto::{binary::Base64Url, messages::Entity},
};

const STREAM_BATCH_SIZE: u64 = 1000;
pub(crate) const DEFAULT_HOST: &str = "https://app.tuta.com";

#[derive(Debug, Clone)]
pub(crate) struct Client {
    inner: reqwest::Client,
}

impl Client {
    pub(crate) fn try_new() -> Result<Self> {
        let inner = reqwest::Client::builder()
            .hickory_dns(true)
            .http2_adaptive_window(true)
            .http2_prior_knowledge()
            .https_only(true)
            .min_tls_version(reqwest::tls::Version::TLS_1_3)
            .user_agent(APP_USER_AGENT)
            .build()
            .context("set up HTTPs client")?;

        Ok(Self { inner })
    }

    pub(crate) fn stream<Resp>(
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
                        .do_json::<(), Vec<Resp>>(Request {
                            method: Method::GET,
                            host: DEFAULT_HOST,
                            prefix: Prefix::Tutanota,
                            path: &path,
                            data: &(),
                            access_token: access_token.as_ref().as_ref(),
                            query: &[
                                ("start", &state.next_start),
                                ("count", &STREAM_BATCH_SIZE.to_string()),
                                ("reverse", "false"),
                            ],
                        })
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

    pub(crate) async fn do_json<Req, Resp>(&self, r: Request<'_, Req>) -> Result<Resp>
    where
        Req: serde::Serialize + Sync,
        Resp: DeserializeOwned,
    {
        let resp = self
            .do_request(r)
            .await?
            .json::<Resp>()
            .await
            .context("fetch JSON response")?;

        Ok(resp)
    }

    pub(crate) async fn do_request<Req>(&self, r: Request<'_, Req>) -> Result<Response>
    where
        Req: serde::Serialize + Sync,
    {
        let Request {
            method,
            host,
            prefix,
            path,
            data,
            access_token,
            query,
        } = r;
        debug!(%method, prefix=prefix.str(), path, "service request",);

        let mut req = self
            .inner
            .request(method, format!("{}/rest/{}/{}", host, prefix.str(), path));

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

#[derive(Debug, Clone, Copy)]
pub(crate) enum Prefix {
    Tutanota,
    Storage,
    Sys,
}

impl Prefix {
    fn str(&self) -> &'static str {
        match self {
            Self::Tutanota => "tutanota",
            Self::Storage => "storage",
            Self::Sys => "sys",
        }
    }
}

pub(crate) struct Request<'a, Req>
where
    Req: serde::Serialize + Sync,
{
    pub(crate) method: Method,
    pub(crate) host: &'a str,
    pub(crate) prefix: Prefix,
    pub(crate) path: &'a str,
    pub(crate) data: &'a Req,
    pub(crate) access_token: Option<&'a Base64Url>,
    pub(crate) query: &'a [(&'a str, &'a str)],
}

impl<'a, Req> Request<'a, Req>
where
    Req: serde::Serialize + Sync,
{
    pub(crate) fn new(prefix: Prefix, path: &'a str, data: &'a Req) -> Self {
        Self {
            method: Method::GET,
            host: DEFAULT_HOST,
            prefix,
            path,
            data,
            access_token: None,
            query: &[],
        }
    }
}
