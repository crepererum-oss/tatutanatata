use std::path::Path;

use anyhow::{Context, Result};
use clap::Parser;
use futures::future::BoxFuture;
use thirtyfour::{
    common::capabilities::firefox::FirefoxPreferences, DesiredCapabilities, WebDriver,
};
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
pub async fn run_webdriver<F>(config: WebdriverCLIConfig, storage_path: &Path, f: F) -> Result<()>
where
    for<'a> F: FnOnce(&'a WebDriver) -> BoxFuture<'a, Result<()>>,
{
    let mut prefs = FirefoxPreferences::new();
    prefs
        .set("browser.download.folderList", 2)
        .context("set pref")?;
    prefs
        .set("browser.download.manager.showWhenStarting", false)
        .context("set pref")?;
    prefs
        .set("browser.download.dir", storage_path.display().to_string())
        .context("set pref")?;
    prefs
        .set(
            "browser.helperApps.neverAsk.saveToDisk",
            "application/octet-stream",
        )
        .context("set pref")?;

    let mut caps = DesiredCapabilities::firefox();
    caps.set_preferences(prefs).context("set preferences")?;
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
