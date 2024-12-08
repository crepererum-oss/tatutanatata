use std::path::Path;

use anyhow::{Context, Result};
use tokio::{fs::OpenOptions, io::AsyncWriteExt};

use crate::retry::retry;

pub(crate) async fn write_to_file(content: &[u8], path: &Path) -> Result<()> {
    let tmp_path = path.with_extension(".part");
    let mut f = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(&tmp_path)
        .await
        .context("open temp file")?;

    f.write_all(content).await.context("write to temp file")?;
    f.shutdown().await.context("close temp file")?;

    rename(&tmp_path, path).await.context("rename")?;

    Ok(())
}

async fn rename(old: &Path, new: &Path) -> Result<()> {
    // some file systems like SMB may not sync immediately and return "not found" shortly after file
    // creation
    retry(
        "rename file",
        || async move { tokio::fs::rename(old, new).await },
        |e| e.kind() == std::io::ErrorKind::NotFound,
    )
    .await
}

pub(crate) fn escape_file_string(s: &str) -> String {
    s.chars()
        .filter(|c| matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9' | ' '))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_file_string() {
        assert_eq!(escape_file_string(""), "");
        assert_eq!(escape_file_string("azaZ09 "), "azaZ09 ");
        assert_eq!(escape_file_string("fOo1!@/\\bar19"), "fOo1bar19");
    }
}
