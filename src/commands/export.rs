use std::{path::Path, time::Duration};

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

pub async fn export(
    config: ExportCLIConfig,
    storage_folder: &Path,
    webdriver: &WebDriver,
) -> Result<()> {
    navigate_to_folder(&config.folder, webdriver)
        .await
        .context("navigate to folder")?;

    let mail_list = get_mail_list(webdriver).await.context("get mail-list")?;
    let list_elements = mail_list
        .find_all(By::Tag("li"))
        .await
        .context("get list elements")?;

    for li in list_elements {
        if !li.is_displayed().await.context("is displayed")? {
            continue;
        }

        li.click().await.context("click at list element")?;

        let mut action_bar = webdriver
            .find_all(By::ClassName("action-bar"))
            .await
            .context("find action-bar")?;
        ensure!(
            action_bar.len() == 1,
            "should have exactly one action bar but found {}",
            action_bar.len()
        );
        let action_bar = action_bar.remove(0);

        let buttons = action_bar
            .find_all(By::Tag("button"))
            .await
            .context("find buttons")?;
        let mut more_button = None;
        for button in buttons {
            if let Some("More") = button
                .attr("title")
                .await
                .context("element attr")?
                .as_deref()
            {
                ensure!(more_button.is_none(), "multiple more buttons");
                more_button = Some(button);
            }
        }
        let more_button = more_button.ok_or_else(|| anyhow!("no more button"))?;

        more_button.click().await.context("click more button")?;

        let mut dropdown_panel = webdriver
            .find_all(By::ClassName("dropdown-panel"))
            .await
            .context("find dropdown-panel")?;
        ensure!(
            dropdown_panel.len() == 1,
            "should have exactly one dropdown panel but found {}",
            dropdown_panel.len()
        );
        let dropdown_panel = dropdown_panel.remove(0);

        let buttons = dropdown_panel
            .find_all(By::Tag("button"))
            .await
            .context("find buttons")?;
        let mut export_button = None;
        for button in buttons {
            let mut text_ellipsis = button
                .find_all(By::ClassName("text-ellipsis"))
                .await
                .context("find text-ellipsis")?;
            ensure!(
                text_ellipsis.len() == 1,
                "should have exactly one text-ellipsis but found {}",
                text_ellipsis.len()
            );
            let text_ellipsis = text_ellipsis.remove(0);

            if text_ellipsis.text().await.context("element text")? == "Export" {
                ensure!(export_button.is_none(), "multiple export buttons");
                export_button = Some(button);
            }
        }
        let export_button = export_button.ok_or_else(|| anyhow!("no export button"))?;

        let n_files_pre = count_files(storage_folder).await.context("count files")?;

        export_button.click().await.context("click export button")?;

        tokio::time::timeout(Duration::from_secs(20), async {
            loop {
                if count_files(storage_folder).await.context("count files")? != n_files_pre {
                    return Ok::<_, anyhow::Error>(());
                }

                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        })
        .await
        .context("timeout waiting for exported file")??;

        ensure_modal_is_closed(webdriver)
            .await
            .context("close modal")?;
    }

    Ok(())
}

async fn count_files(path: &Path) -> Result<usize> {
    let mut files = tokio::fs::read_dir(path).await?;
    let mut n = 0;
    while files.next_entry().await?.is_some() {
        n += 1;
    }

    Ok(n)
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

async fn ensure_modal_is_closed(webdriver: &WebDriver) -> Result<()> {
    tokio::time::timeout(Duration::from_secs(20), async {
        loop {
            let mut modal = webdriver
                .find_all(By::Id("modal"))
                .await
                .context("find modal")?;
            ensure!(
                modal.len() == 1,
                "should have exactly one modal but found {}",
                modal.len()
            );
            let modal = modal.remove(0);

            if !modal.is_displayed().await.context("modal displayed")? {
                return Ok::<_, anyhow::Error>(());
            }

            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    })
    .await
    .context("modal not closing in time")??;

    Ok(())
}
