use std::time::Duration;

use anyhow::{anyhow, ensure, Context, Result};
use clap::Parser;
use thirtyfour::{By, WebDriver, WebElement};
use tracing::debug;

use super::list_folders::list_folders;

/// Export CLI config.
#[derive(Debug, Parser)]
pub struct ExportCLIConfig {
    /// Folder
    #[clap(long)]
    folder: String,
}

pub async fn export(config: ExportCLIConfig, webdriver: &WebDriver) -> Result<()> {
    navigate_to_folder(&config.folder, webdriver)
        .await
        .context("navigate to folder")?;

    webdriver
        .screenshot(std::path::Path::new("./foo.png"))
        .await
        .unwrap();

    Ok(())
}

async fn get_mail_list(webdriver: &WebDriver) -> Result<WebElement> {
    let mut mail_list = webdriver
        .find_all(By::ClassName("mail-list"))
        .await
        .context("find mail list")?;

    ensure!(
        mail_list.len() == 1,
        "should have exactly one mail list but found {}",
        mail_list.len()
    );
    let mail_list = mail_list.remove(0);

    let tag_name = mail_list.tag_name().await.context("get tag name")?;
    ensure!(
        tag_name == "ul",
        "mail-list should be a list but is {}",
        tag_name
    );

    Ok(mail_list)
}

async fn is_mail_list_loading(webdriver: &WebDriver) -> Result<bool> {
    let mail_list = get_mail_list(webdriver).await.context("get mail-list")?;

    let mut progress_icon = mail_list
        .find_all(By::ClassName("icon-progress"))
        .await
        .context("find progress icon")?;
    ensure!(
        progress_icon.len() == 1,
        "should have exactly one progress icon but found {}",
        progress_icon.len()
    );
    let progress_icon = progress_icon.remove(0);
    progress_icon.is_displayed().await.context("is displayed")
}

async fn navigate_to_folder(folder: &str, webdriver: &WebDriver) -> Result<()> {
    for (anchor, title) in list_folders(webdriver).await.context("list folders")? {
        if title == folder {
            anchor.click().await.context("clicking folder link")?;

            tokio::time::timeout(Duration::from_secs(20), async {
                loop {
                    if !is_mail_list_loading(webdriver)
                        .await
                        .context("is mail-list loading?")?
                    {
                        return Ok::<_, anyhow::Error>(());
                    }

                    debug!("folder still loading");
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            })
            .await
            .context("wait for folder update")??;

            return Ok(());
        }
    }

    Err(anyhow!("folder not found"))
}
