use std::{future::Future, sync::Arc};

use anyhow::{Context, Result};
use futures::Stream;
use reqwest::{Method, Response, StatusCode};
use serde::de::DeserializeOwned;
use tokio::{
    sync::mpsc::{channel, Receiver},
    task::JoinSet,
};
use tracing::{debug, warn};

use crate::{
    constants::APP_USER_AGENT,
    proto::{binary::Base64Url, messages::Entity},
};

const STREAM_BATCH_SIZE: u64 = 1000;
const STREAM_BUFFER_SIZE: u64 = 4 * STREAM_BATCH_SIZE;
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
        Resp: DeserializeOwned + Entity + Send + 'static,
    {
        let (tx, rx) = channel(STREAM_BUFFER_SIZE as usize);

        let path = Arc::new(path.to_owned());
        let access_token = Arc::new(access_token.cloned());
        let this = self.clone();
        let mut fetch_task = JoinSet::new();
        fetch_task.spawn(async move {
            let mut next_start = "------------".to_owned();

            loop {
                debug!(
                    path = path.as_str(),
                    start = next_start.as_str(),
                    "fetch new page",
                );

                let res = this
                    .do_json::<(), Vec<Resp>>(Request {
                        method: Method::GET,
                        host: DEFAULT_HOST,
                        prefix: Prefix::Tutanota,
                        path: &path,
                        data: &(),
                        access_token: access_token.as_ref().as_ref(),
                        query: &[
                            ("start", &next_start),
                            ("count", &STREAM_BATCH_SIZE.to_string()),
                            ("reverse", "false"),
                        ],
                    })
                    .await
                    .context("fetch next page");

                match res {
                    Ok(elements) => {
                        match elements.last() {
                            None => {
                                // reached end
                                return;
                            }
                            Some(o) => {
                                o.id().to_owned().clone_into(&mut next_start);
                            }
                        }

                        for o in elements {
                            if tx.send(Ok(o)).await.is_err() {
                                // receiver gone
                                return;
                            }
                        }
                    }
                    Err(e) => {
                        // it's OK if the receiver is gone, we exit anyways
                        tx.send(Err(e)).await.ok();
                        return;
                    }
                }
            }
        });
        let state = StreamState { rx, fetch_task };

        futures::stream::unfold(state, move |mut state: StreamState<Resp>| async move {
            state.rx.recv().await.map(|next| (next, state))
        })
    }

    pub(crate) async fn do_json<Req, Resp>(&self, r: Request<'_, Req>) -> Result<Resp>
    where
        Req: serde::Serialize + Sync,
        Resp: DeserializeOwned,
    {
        let s = retry(|| async { self.do_request(r.clone()).await?.text().await }).await?;

        let jd = &mut serde_json::Deserializer::from_str(&s);
        let res: Result<Resp, _> = serde_path_to_error::deserialize(jd);

        res.with_context(|| {
            println!();
            println!();
            println!("{s}");
            println!();
            println!();
            format!("deserialize JSON for `{}`", std::any::type_name::<Resp>())
        })
    }

    pub(crate) async fn do_bytes<Req>(&self, r: Request<'_, Req>) -> Result<Vec<u8>>
    where
        Req: serde::Serialize + Sync,
    {
        let b = retry(|| async { self.do_request(r.clone()).await?.bytes().await }).await?;

        Ok(b.to_vec())
    }

    pub(crate) async fn do_no_response<Req>(&self, r: Request<'_, Req>) -> Result<()>
    where
        Req: serde::Serialize + Sync,
    {
        retry(|| async { self.do_request(r.clone()).await }).await?;

        Ok(())
    }

    async fn do_request<Req>(&self, r: Request<'_, Req>) -> Result<Response, reqwest::Error>
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
            .await?
            .error_for_status()?;

        Ok(resp)
    }
}

struct StreamState<T> {
    rx: Receiver<Result<T>>,
    #[allow(dead_code)]
    fetch_task: JoinSet<()>,
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

impl<'a, Req> Clone for Request<'a, Req>
where
    Req: serde::Serialize + Sync,
{
    fn clone(&self) -> Self {
        Self {
            method: self.method.clone(),
            host: self.host,
            prefix: self.prefix,
            path: self.path,
            data: self.data,
            access_token: self.access_token,
            query: self.query,
        }
    }
}

async fn retry<F, Fut, T>(action: F) -> Result<T, reqwest::Error>
where
    F: Fn() -> Fut + Send,
    Fut: Future<Output = Result<T, reqwest::Error>> + Send,
{
    // WARNING: The exponential config is somewhat weird. `from_millis(base).factor(factor)` means
    //          `base^retry * factor`.
    //          Also see https://github.com/srijs/rust-tokio-retry/issues/22 .
    let strategy = tokio_retry::strategy::ExponentialBackoff::from_millis(2)
        .factor(500)
        .map(tokio_retry::strategy::jitter);

    let condition = |e: &reqwest::Error| {
        if e.is_connect() || e.is_timeout() {
            return true;
        }

        if let Some(status) = e.status() {
            if status.is_server_error()
                || (status == StatusCode::REQUEST_TIMEOUT)
                || (status == StatusCode::TOO_MANY_REQUESTS)
            {
                return true;
            }
        }

        false
    };
    let condition = move |e: &reqwest::Error| {
        let should_retry = condition(e);
        if should_retry {
            warn!(%e, "retry client error");
        }
        should_retry
    };

    tokio_retry::RetryIf::spawn(strategy, action, condition).await
}
