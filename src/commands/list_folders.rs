use anyhow::{anyhow, ensure, Context, Result};
use thirtyfour::{By, WebDriver};
use tracing::debug;

pub async fn list_folders(webdriver: &WebDriver) -> Result<Vec<String>> {
    let mut folder_column = webdriver
        .find_all(By::ClassName("folder-column"))
        .await
        .context("find folder column")?;
    ensure!(folder_column.len() == 1, "has exactly 1 folder column");
    let folder_column = folder_column.remove(0);
    debug!("found folder column");

    let rows = folder_column
        .find_all(By::ClassName("folder-row"))
        .await
        .context("find folder rows")?;
    debug!("found folder rows");

    let mut folders = Vec::with_capacity(rows.len());
    for row in rows {
        let mut anchor = row
            .find_all(By::Tag("a"))
            .await
            .context("find folder anchor")?;
        if anchor.is_empty() {
            continue;
        }

        ensure!(
            anchor.len() == 1,
            "has at most 1 folder anchor in a row, but has {}",
            anchor.len()
        );
        let anchor = anchor.remove(0);

        let title = anchor
            .attr("title")
            .await
            .context("element attr")?
            .ok_or_else(|| anyhow!("anchor has no title"))?;
        folders.push(title);
    }

    ensure!(!folders.is_empty(), "list of folders should never be empty");
    debug!("found folders");

    Ok(folders)
}
