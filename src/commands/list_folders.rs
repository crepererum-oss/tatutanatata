use std::collections::HashSet;

use anyhow::{anyhow, ensure, Context, Result};
use thirtyfour::{By, WebDriver, WebElement};
use tracing::debug;

use crate::thirtyfour_util::FindExt;

pub async fn list_folders(webdriver: &WebDriver) -> Result<Vec<(WebElement, String)>> {
    let folder_column = webdriver
        .find_one(By::ClassName("folder-column"))
        .await
        .context("find folder column")?;
    debug!("found folder column");

    let rows = folder_column
        .find_all(By::ClassName("folder-row"))
        .await
        .context("find folder rows")?;
    debug!("found folder rows");

    let mut folders = Vec::with_capacity(rows.len());
    let mut seen = HashSet::new();
    for row in rows {
        let Some(anchor )= row
            .find_at_most_one(By::Tag("a"))
            .await
            .context("find folder anchor")? else {
                continue
            };

        let title = anchor
            .attr("title")
            .await
            .context("element attr")?
            .ok_or_else(|| anyhow!("anchor has no title"))?;

        ensure!(seen.insert(title.clone()), "duplicate folder: {}", title);

        folders.push((anchor, title));
    }

    ensure!(!folders.is_empty(), "list of folders should never be empty");
    debug!("found folders");

    Ok(folders)
}
