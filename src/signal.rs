use std::future::Future;

use anyhow::{Context, Result};
use tokio::signal::unix::SignalKind;
use tracing::warn;

pub(crate) trait FutureSignalExt {
    async fn cancel_on_signal(self) -> Result<()>;
}

impl<F> FutureSignalExt for F
where
    F: Future<Output = Result<()>> + Send,
{
    async fn cancel_on_signal(self) -> Result<()> {
        let mut sigterm_listener =
            tokio::signal::unix::signal(SignalKind::terminate()).context("listen for SIGTERM")?;
        let ctrc_listener = tokio::signal::ctrl_c();

        tokio::select! {
            _ = sigterm_listener.recv() => {
                warn!("terminated by SIGTERM");
                Ok(())
            }
            res = ctrc_listener => {
                res.context("listen for CTRL-C")?;
                warn!("terminated by CTRL-C");
                Ok(())
            }
            res = self => {
                res
            }
        }
    }
}
