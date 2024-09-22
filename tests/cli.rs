#![allow(unused_crate_dependencies)]

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use assert_cmd::Command;
use tempfile::TempDir;

#[test]
fn test_help_arg() {
    let mut cmd = cmd();
    cmd.arg("--help").assert().success();
}

#[test]
fn test_help_cmd() {
    let mut cmd = cmd();
    cmd.arg("help").assert().success();
}

#[test]
fn test_version_arg() {
    let mut cmd = cmd();
    cmd.arg("--version").assert().success();
}

fn cmd() -> Command {
    Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap()
}

fn read_files(path: &Path) -> HashMap<String, String> {
    let mut out = HashMap::default();

    for f in std::fs::read_dir(path).unwrap() {
        let f = f.unwrap();
        assert!(f.file_type().unwrap().is_file());
        out.insert(
            f.path().file_name().unwrap().to_str().unwrap().to_owned(),
            std::fs::read_to_string(f.path()).unwrap(),
        );
    }

    out
}

mod integration {
    use super::*;

    #[test]
    fn test_debug_dump_json() {
        let tmp_dir = TempDir::new().unwrap();

        // use path that does NOT exist
        let dump_dir = tmp_dir.path().join("json");

        let mut cmd = cmd();
        cmd.arg("-vv")
            .arg("--debug-dump-json-to")
            .arg(&dump_dir)
            .arg("list-folders")
            .assert()
            .success();

        assert!(std::fs::read_dir(dump_dir).unwrap().count() > 0);
    }

    #[test]
    fn test_list_folders() {
        let mut cmd = cmd();
        let res = cmd.arg("-vv").arg("list-folders").assert().success();
        let stdout = String::from_utf8(res.get_output().stdout.clone()).unwrap();

        insta::assert_snapshot!(stdout, @r###"
        Inbox
        Sent
        Trash
        Archive
        Spam
        Draft
        fooooo
        "###);
    }

    #[test]
    fn test_download() {
        let actual_path = TempDir::new().unwrap();

        let mut expected_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        expected_path.push("tests");
        expected_path.push("reference");

        let mut cmd = cmd();
        cmd.arg("-vv")
            .arg("download")
            .arg("--folder=fooooo")
            .arg("--path")
            .arg(actual_path.path())
            .assert()
            .success();

        let actual = read_files(actual_path.path());
        let expected = read_files(&expected_path);

        let mut actual_files = actual.keys().collect::<Vec<_>>();
        actual_files.sort();
        let mut expected_files = expected.keys().collect::<Vec<_>>();
        expected_files.sort();
        assert_eq!(actual_files, expected_files);

        for fname in actual_files {
            let actual_content = actual.get(fname).unwrap();
            let expected_content = expected.get(fname).unwrap();
            similar_asserts::assert_eq!(actual_content, expected_content);
        }
    }
}
