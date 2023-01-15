use std::time::Duration;

use anyhow::{Context, Result};
use clap::Parser;
use thirtyfour::{By, WebDriver};
use tracing::debug;

use crate::thirtyfour_util::FindExt;

/// Login CLI config.
#[derive(Debug, Parser)]
pub struct LoginCLIConfig {
    /// Username
    #[clap(long, env = "TUTANOTA_CLI_USERNAME")]
    username: String,

    /// Password
    #[clap(long, env = "TUTANOTA_CLI_PASSWORD")]
    password: String,
}

/// Perform tutanota webinterface login.
pub async fn perform_login(config: LoginCLIConfig, webdriver: &WebDriver) -> Result<()> {
    webdriver
        .goto("https://mail.tutanota.com")
        .await
        .context("go to webinterface")?;
    debug!("navigated to login page");

    let input_username = webdriver
        .find_one_with_attr(By::Tag("input"), "autocomplete", "email")
        .await
        .context("find username input")?;
    let input_password = webdriver
        .find_one_with_attr(By::Tag("input"), "autocomplete", "current-password")
        .await
        .context("find password input")?;
    debug!("found username and password inputs");

    input_username
        .send_keys(config.username)
        .await
        .context("enter username")?;
    input_password
        .send_keys(config.password)
        .await
        .context("enter password")?;
    debug!("entered username and password");

    let login_button = webdriver
        .find_one_with_attr(By::Tag("button"), "title", "Log in")
        .await
        .context("find login button")?;
    debug!("found login button");

    login_button.click().await.context("click login button")?;
    debug!("clicked login button, waiting for login");

    tokio::time::timeout(Duration::from_secs(20), async {
        loop {
            if has_new_email_button(webdriver)
                .await
                .context("search new-email button")?
            {
                return Ok::<_, anyhow::Error>(());
            }

            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    })
    .await
    .context("wait for login")??;
    debug!("login done");

    Ok(())
}

async fn has_new_email_button(webdriver: &WebDriver) -> Result<bool> {
    let buttons = webdriver
        .find_all(By::Tag("button"))
        .await
        .context("find button elements")?;

    for button in buttons {
        if let Some("New email") = button
            .attr("title")
            .await
            .context("element attr")?
            .as_deref()
        {
            return Ok(true);
        }
    }

    Ok(false)
}
