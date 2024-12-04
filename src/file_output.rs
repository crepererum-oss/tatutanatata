use std::path::Path;

use anyhow::{Context, Result};
use tokio::{fs::OpenOptions, io::AsyncWriteExt};
use tracing::warn;

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

async fn rename(old: &Path, new: &Path) -> Result<(), std::io::Error> {
    // some file systems like SMB may not sync immediately and return "not found" shortly after file
    // creation

    // WARNING: The exponential config is somewhat weird. `from_millis(base).factor(factor)` means
    //          `base^retry * factor`.
    //          Also see https://github.com/srijs/rust-tokio-retry/issues/22 .
    let strategy = tokio_retry::strategy::ExponentialBackoff::from_millis(2)
        .factor(500)
        .map(tokio_retry::strategy::jitter)
        .take(10);

    let condition = |e: &std::io::Error| e.kind() == std::io::ErrorKind::NotFound;
    let condition = |e: &std::io::Error| {
        let should_retry = condition(e);
        if should_retry {
            warn!(%e, "retry rename");
        }
        should_retry
    };

    let action = || async move { tokio::fs::rename(old, new).await };

    tokio_retry::RetryIf::spawn(strategy, action, condition).await
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
