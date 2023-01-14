use anyhow::{Context, Result};
use clap::Parser;
use futures::future::BoxFuture;
use thirtyfour::{DesiredCapabilities, WebDriver};
use tracing::debug;

use crate::error::MultiResultExt;

/// Webdriver CLI config.
#[derive(Debug, Parser)]
pub struct WebdriverCLIConfig {
    /// Webdriver port.
    #[clap(long, default_value_t = 4444)]
    webdriver_port: u16,
}

/// Run given async method with the given webdriver.
///
/// This ensures that the webdriver is shut down after the given future finishes.
pub async fn run_webdriver<F>(config: WebdriverCLIConfig, f: F) -> Result<()>
where
    for<'a> F: FnOnce(&'a WebDriver) -> BoxFuture<'a, Result<()>>,
{
    let mut caps = DesiredCapabilities::firefox();
    caps.set_headless().context("enable headless")?;

    let addr = format!("http://localhost:{}", config.webdriver_port);
    let driver = WebDriver::new(&addr, caps)
        .await
        .context("webdriver setup")?;
    debug!("webdriver setup done");

    let res = f(&driver).await;

    driver
        .quit()
        .await
        .context("webdriver shutdown")
        .combine(res)
        .map_err(|e| e.into_anyhow())?;
    debug!("webdriver shutdown done");

    Ok(())
}
