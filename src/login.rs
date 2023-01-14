use std::time::Duration;

use anyhow::{anyhow, ensure, Context, Result};
use clap::Parser;
use thirtyfour::{By, WebDriver};
use tracing::debug;

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

    let inputs = webdriver
        .find_all(By::Tag("input"))
        .await
        .context("find input elements")?;
    let mut input_username = None;
    let mut input_password = None;
    for input in inputs {
        match input
            .attr("autocomplete")
            .await
            .context("element attr")?
            .as_deref()
        {
            Some("email") => {
                ensure!(input_username.is_none(), "multiple username inputs");
                input_username = Some(input);
            }
            Some("current-password") => {
                ensure!(input_password.is_none(), "multiple password inputs");
                input_password = Some(input);
            }
            _ => {}
        }
    }

    let input_username = input_username.ok_or_else(|| anyhow!("no username input"))?;
    let input_password = input_password.ok_or_else(|| anyhow!("no password input"))?;
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

    let buttons = webdriver
        .find_all(By::Tag("button"))
        .await
        .context("find button elements")?;
    let mut login_button = None;
    for button in buttons {
        if let Some("Log in") = button
            .attr("title")
            .await
            .context("element attr")?
            .as_deref()
        {
            ensure!(login_button.is_none(), "multiple login buttons");
            login_button = Some(button);
        }
    }
    let login_button = login_button.ok_or_else(|| anyhow!("no login button"))?;
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
