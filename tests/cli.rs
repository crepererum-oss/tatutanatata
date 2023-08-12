use std::sync::{LockResult, Mutex, MutexGuard};

use assert_cmd::Command;
use predicates::prelude::*;

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
#[ignore]
fn test_export() {
    let _guard = webdriver_mutex();
    let mut cmd = cmd();
    cmd.arg("--screenshot-on-failure")
        .arg("--screenshot-path=test_export.png")
        .arg("-vv")
        .arg("export")
        .arg("--folder=Archive")
        .assert()
        .success();
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
