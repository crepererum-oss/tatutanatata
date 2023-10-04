use std::{
    path::PathBuf,
    sync::{LockResult, Mutex, MutexGuard},
};

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

/// We can only have a single webdriver session.
static WEBDRIVER_MUTEX: Mutex<()> = Mutex::new(());

#[test]
fn test_help() {
    let mut cmd = cmd();
    cmd.arg("--help").assert().success();
}

#[test]
fn test_list_folders() {
    let _guard = webdriver_mutex();
    let mut cmd = cmd();
    cmd.arg("--screenshot-on-failure")
        .arg("--screenshot-path=test_list_folders.png")
        .arg("-vv")
        .arg("list-folders")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            ["Inbox", "Drafts", "Sent", "Trash", "Archive", "Spam"].join("\n"),
        ));
}

#[test]
fn test_export() {
    let _guard = webdriver_mutex();
    let storage_folder = storage_folder();

    let mut cmd = cmd();
    cmd.arg("--screenshot-on-failure")
        .arg("--screenshot-path=test_export.png")
        .arg("--storage-folder")
        .arg(storage_folder.display().to_string())
        .arg("-vv")
        .arg("export")
        .arg("--folder=Archive")
        .assert()
        .success();

    let n_files = std::fs::read_dir(storage_folder)
        .expect("can read output dir")
        .count();
    assert_eq!(n_files, 3);
}

fn cmd() -> Command {
    Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap()
}

fn webdriver_mutex() -> MutexGuard<'static, ()> {
    match WEBDRIVER_MUTEX.lock() {
        LockResult::Ok(guard) => guard,
        // poisoned locks are OK
        LockResult::Err(e) => e.into_inner(),
    }
}

fn storage_folder() -> PathBuf {
    if let Ok(path) = std::env::var("TEST_STORAGE_DIR") {
        return path.try_into().expect("valid path");
    }

    tempdir().expect("can create temp dir").into_path()
}
