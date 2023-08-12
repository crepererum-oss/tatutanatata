use std::path::{Path, PathBuf};

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
    /// Create screenshot on failure.
    #[clap(long, default_value_t = false)]
    screenshot_on_failure: bool,

    /// Path for screenshot.
    #[clap(long, default_value = "./screenshot.png")]
    screenshot_path: PathBuf,

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
    driver.maximize_window().await.context("maximize window")?;
    debug!("webdriver setup done");

    let res_f = f(&driver).await;

    let res_screenshot = if res_f.is_err() && config.screenshot_on_failure {
        driver
            .screenshot(&config.screenshot_path)
            .await
            .context("create screenshot")
    } else {
        Ok(())
    };

    let res_shutdown = driver.quit().await.context("webdriver shutdown");

    res_shutdown
        .combine(res_screenshot)
        .combine(res_f)
        .map_err(|e| e.into_anyhow())?;
    debug!("webdriver shutdown done");

    Ok(())
}
