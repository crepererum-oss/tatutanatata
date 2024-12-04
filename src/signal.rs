use std::future::Future;

use anyhow::{Context, Result};
use tracing::warn;

pub(crate) trait FutureSignalExt {
    async fn cancel_on_signal(self) -> Result<()>;
}

impl<F> FutureSignalExt for F
where
    F: Future<Output = Result<()>> + Send,
{
    async fn cancel_on_signal(self) -> Result<()> {
        let signal_listener = wait_signal()?;
        let ctrc_listener = tokio::signal::ctrl_c();

        tokio::select! {
            sig = signal_listener => {
                warn!("terminated by {}", sig);
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

#[cfg(unix)]
fn wait_signal() -> Result<impl Future<Output = &'static str>> {
    use tokio::signal::unix::SignalKind;

    let mut sigterm_listener =
        tokio::signal::unix::signal(SignalKind::terminate()).context("listen for SIGTERM")?;

    Ok(async move {
        sigterm_listener.recv().await;
        "SIGTERM"
    })
}

#[cfg(windows)]
fn wait_signal() -> Result<impl Future<Output = &'static str>> {
    Ok(futures::future::pending())
}
