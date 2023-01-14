use std::{
    io::ErrorKind,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use clap::Parser;

/// Storage CLI config.
#[derive(Debug, Parser)]
pub struct StorageCLIConfig {
    /// Storage folder.
    #[clap(long, default_value = "./out")]
    storage_folder: PathBuf,
}

pub async fn setup_storage(config: StorageCLIConfig) -> Result<PathBuf> {
    match use_existing_folder(&config.storage_folder).await {
        Ok(path) => Ok(path),
        Err(e) if e.kind() == ErrorKind::NotFound => use_new_folder(&config.storage_folder)
            .await
            .context("create new dir"),
        Err(e) => Err(e).context("cannot use storage path"),
    }
}

async fn use_new_folder(path: &Path) -> Result<PathBuf, std::io::Error> {
    tokio::fs::create_dir_all(path).await?;
    tokio::fs::canonicalize(path).await
}

async fn use_existing_folder(path: &Path) -> Result<PathBuf, std::io::Error> {
    let path = tokio::fs::canonicalize(path).await?;

    let is_empty = is_empty(&path).await?;

    if !is_empty {
        return Err(std::io::Error::new(ErrorKind::Other, "folder not empty"));
    }

    Ok(path)
}

async fn is_empty(path: &Path) -> Result<bool, std::io::Error> {
    Ok(tokio::fs::read_dir(path)
        .await?
        .next_entry()
        .await?
        .is_none())
}
