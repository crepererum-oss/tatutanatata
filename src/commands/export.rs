use std::{collections::HashSet, path::Path, time::Duration};

use anyhow::{anyhow, ensure, Context, Result};
use clap::Parser;
use thirtyfour::{By, WebDriver, WebElement};
use tracing::{debug, info};

use crate::thirtyfour_util::FindExt;

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

    let mut seen = HashSet::new();

    loop {
        let n_seen = seen.len();
        let (seen2, list_updated) = export_round(storage_folder, webdriver, seen).await?;
        seen = seen2;
        if seen.len() == n_seen && !list_updated {
            break;
        }
        ensure_list_is_ready(webdriver).await?;
    }

    info!(n = seen.len(), "exported folder");

    Ok(())
}

async fn export_round(
    storage_folder: &Path,
    webdriver: &WebDriver,
    mut seen: HashSet<u64>,
) -> Result<(HashSet<u64>, bool)> {
    let mail_list = get_mail_list(webdriver)
        .await
        .context("get mail-list")?
        .context("mailing list no longer loading")?;

    let list_height = style_height(&mail_list).await.context("list height")?;

    let list_elements = mail_list
        .find_all(By::Tag("li"))
        .await
        .context("get list elements")?;
    let list_size = list_elements.len();
    info!(list_size, list_height, "found mail list");

    let mut list_elements_with_y = Vec::with_capacity(list_elements.len());
    for li in list_elements {
        if let Some(translate_y) = style_translate_y(&li).await.context("get translateY")? {
            list_elements_with_y.push((li, translate_y));
        }
    }
    list_elements_with_y.sort_by_key(|(_li, y)| *y);

    if list_elements_with_y.is_empty() {
        info!("empty list");
        return Ok((seen, false));
    }

    for (li, y) in &list_elements_with_y {
        if !seen.insert(*y) {
            continue;
        }

        info!(list_size, y, "handle entry");

        let checkbox = li
            .find_one(By::ClassName("checkbox"))
            .await
            .context("find list entry checkbox")?;

        checkbox.click().await.context("click at checkbox")?;
        export_current_mail(storage_folder, webdriver)
            .await
            .context("export current main")?;
        checkbox.click().await.context("click at checkbox")?;

        let list_height2 = style_height(&mail_list).await.context("list height")?;
        let first_element_y = style_translate_y(&list_elements_with_y[0].0)
            .await
            .context("first element y")?
            .ok_or_else(|| anyhow!("first element lost y"))?;
        if list_height != list_height2 || list_elements_with_y[0].1 != first_element_y {
            info!("list updated");
            return Ok((seen, true));
        }
    }

    Ok((seen, false))
}

async fn export_current_mail(storage_folder: &Path, webdriver: &WebDriver) -> Result<()> {
    let action_bar = webdriver
        .find_one(By::ClassName("scrollbar-gutter-stable-or-fallback"))
        .await
        .context("find action-bar")?;

    let export_button = tokio::time::timeout(Duration::from_secs(20), async {
        loop {
            match action_bar
                .find_one_with_attr(By::Tag("button"), "title", "Export")
                .await
                .context("find export button")
            {
                Ok(b) => {
                    return b;
                }
                Err(e) => {
                    debug!(%e, "cannot find export button");
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            }
        }
    })
    .await
    .context("timeout while waiting for the export button")?;

    let n_files_pre = count_files(storage_folder).await.context("count files")?;

    export_button.click().await.context("click export button")?;

    tokio::time::timeout(Duration::from_secs(20), async {
        loop {
            if count_files(storage_folder).await.context("count files")? != n_files_pre {
                return Ok::<_, anyhow::Error>(());
            }

            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    })
    .await
    .context("timeout waiting for exported file")??;

    ensure_modal_is_closed(webdriver)
        .await
        .context("close modal")?;

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

async fn get_mail_list(webdriver: &WebDriver) -> Result<Option<WebElement>> {
    let Some(mail_list) = webdriver
        .find_at_most_one(By::ClassName("list"))
        .await
        .context("find mail list")?
    else {
        return Ok(None);
    };

    let tag_name = mail_list.tag_name().await.context("get tag name")?;
    ensure!(
        tag_name == "ul",
        "mail-list should be a list but is {}",
        tag_name
    );

    Ok(Some(mail_list))
}

async fn is_mail_list_loading(webdriver: &WebDriver) -> Result<bool> {
    let Some(mail_list) = get_mail_list(webdriver).await.context("get mail-list")? else {
        return Ok(true);
    };

    let Some(progress_icon) = mail_list
        .find_at_most_one(By::ClassName("icon-progress"))
        .await
        .context("find progress icon")?
    else {
        return Ok(false);
    };
    progress_icon.is_displayed().await.context("is displayed")
}

async fn navigate_to_folder(folder: &str, webdriver: &WebDriver) -> Result<()> {
    for (anchor, title) in list_folders(webdriver).await.context("list folders")? {
        if title == folder {
            // modal might be left-over from some login dialog, make sure it is gone before we
            // attempt to click any buttons
            ensure_modal_is_closed(webdriver)
                .await
                .context("ensure modal is closed")?;

            anchor.click().await.context("clicking folder link")?;

            ensure_list_is_ready(webdriver)
                .await
                .context("folder ready?")?;

            return Ok(());
        }
    }

    Err(anyhow!("folder not found"))
}

async fn ensure_list_is_ready(webdriver: &WebDriver) -> Result<()> {
    tokio::time::timeout(Duration::from_secs(20), async {
        loop {
            if !is_mail_list_loading(webdriver)
                .await
                .context("is mail-list loading?")?
            {
                return Ok::<_, anyhow::Error>(());
            }

            info!("folder still loading");
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    })
    .await
    .context("wait for folder update")??;

    Ok(())
}

async fn ensure_modal_is_closed(webdriver: &WebDriver) -> Result<()> {
    debug!("ensure modal is closed");

    tokio::time::timeout(Duration::from_secs(20), async {
        loop {
            let modal = webdriver
                .find_one(By::Id("modal"))
                .await
                .context("find modal")?;

            if !modal.is_displayed().await.context("modal displayed")? {
                return Ok::<_, anyhow::Error>(());
            }

            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    })
    .await
    .context("modal not closing in time")??;

    debug!("modal is closed");

    Ok(())
}

async fn style_height(element: &WebElement) -> Result<u64> {
    let style = element
        .attr("style")
        .await
        .context("list height")?
        .ok_or_else(|| anyhow!("no style data found"))?;

    let needle = "height: ";
    let pos = style
        .find(needle)
        .ok_or_else(|| anyhow!("height not found"))?;
    let style = &style[pos + needle.len()..];

    let needle = "px";
    let pos = style.find(needle).ok_or_else(|| anyhow!("px not found"))?;

    style[..pos].parse().context("cannot parse height")
}

async fn style_translate_y(element: &WebElement) -> Result<Option<u64>> {
    let style = element
        .attr("style")
        .await
        .context("list height")?
        .ok_or_else(|| anyhow!("no style data found"))?;

    if style.contains("display: none") {
        return Ok(None);
    }

    let needle = "translateY(";
    let pos = style
        .find(needle)
        .ok_or_else(|| anyhow!("translateY not found: {}", style))?;
    let style = &style[pos + needle.len()..];

    let needle = "px";
    let pos = style.find(needle).ok_or_else(|| anyhow!("px not found"))?;

    let y = style[..pos].parse().context("cannot parse translation")?;
    Ok(Some(y))
}
