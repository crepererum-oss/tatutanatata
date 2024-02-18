#![allow(unused_crate_dependencies)]

use assert_cmd::Command;

#[test]
fn test_help() {
    let mut cmd = cmd();
    cmd.arg("--help").assert().success();
}

#[test]
fn test_list_folders() {
    let mut cmd = cmd();
    let res = cmd.arg("-vv").arg("list-folders").assert().success();
    let stdout = String::from_utf8(res.get_output().stdout.clone()).unwrap();

    insta::assert_display_snapshot!(stdout, @r###"
    Inbox
    Sent
    Trash
    Archive
    Spam
    Draft
    fooooo
    "###);
}

fn cmd() -> Command {
    Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap()
}
