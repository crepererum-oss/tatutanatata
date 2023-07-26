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
    
    /// One-Time-Code
    #[clap(long, default_value_t = 0.to_string(), env = "TUTANOTA_CLI_ONETIMECODE")]
    onetimecode: String,
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
    debug!("clicked login button, checking for one-time-code or login");
    
    // Sloppily re-defined onetimecode var here because of "ownership" issues in the loop (Rust noob)
    let onetimecode = config.onetimecode;
    // loggedin tracks if timeout loop exited due to successful login or due to 2FA detection
    let mut loggedin = false;
    
    // Check if login successful or if 2FA required (Cancel button appears)    
    tokio::time::timeout(Duration::from_secs(20), async {
        loop {
            if has_new_email_button(webdriver)
                .await
                .context("search new-email button")?
            {
            	loggedin = true;
                return Ok::<_, anyhow::Error>(()); // Login successful
            }
            else if has_cancel_button(webdriver)
                .await
                .context("search cancel button")?
            {
            	// Cancel button present means 2FA needed
            	enter_onetimecode(&onetimecode, webdriver)
            	    .await
            	    .context("entering one-time-code and finishing login")?;
            	return Ok::<_, anyhow::Error>(()); // 2FA entered, logging in...
            }

            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    })
    .await
    .context("wait for one-time-code or login")??;
    debug!("wait done");
    
    if loggedin
    {
        return Ok(()); // No 2FA; login successful; done
    }
    
    // Login not completed due to 2FA; wait for login to complete
    tokio::time::timeout(Duration::from_secs(20), async {
        loop {
            if has_new_email_button(webdriver)
                .await
                .context("search new-email button")?
            {
            	loggedin = true;
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

async fn has_cancel_button(webdriver: &WebDriver) -> Result<bool> {
    let buttons = webdriver
        .find_all(By::Tag("button"))
        .await
        .context("find button elements")?;

    for button in buttons {
        if let Some("Cancel") = button
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

async fn enter_onetimecode(onetimecode: &String, webdriver: &WebDriver) -> Result<bool> {
    let input_onetimecode = webdriver
        .find_one_with_attr(By::Tag("input"), "autocomplete", "one-time-code")
        .await
        .context("find one-time-code input");
    debug!("found one-time-code input");
    
    input_onetimecode?
        .send_keys(onetimecode)
        .await
        .context("enter one-time-code")?;
    debug!("entered one-time-code");
    
    let ok_button = webdriver
        .find_one_with_attr(By::Tag("button"), "title", "Ok")
        .await
        .context("find ok button")?;
    debug!("found ok button");

    ok_button.click().await.context("click ok button")?;
    debug!("clicked ok button");
    
    Ok(true)
}
