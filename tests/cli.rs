use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_help() {
    let mut cmd = cmd();
    cmd.arg("--help").assert().success();
}

#[test]
fn test_list_folders() {
    let mut cmd = cmd();
    cmd.arg("-vv")
        .arg("list-folders")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            [
                "Inbox", "Sent", "Trash", "Archive", "Spam", "Draft", "fooooo",
            ]
            .join("\n"),
        ));
}

fn cmd() -> Command {
    Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap()
}
