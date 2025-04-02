use assert_cmd::Command;
use predicates::prelude::*;

fn bin() -> Command {
    Command::cargo_bin("jas").unwrap()
}

#[test]
fn test_install_github() {
    let mut cmd = bin();
    cmd.arg("install")
        .arg("--gh")
        .arg("crate-ci/typos@v1.31.1")
        .arg("--dir")
        .arg("tests")
        .assert()
        .success();
    let path = std::path::Path::new("tests/typos");
    assert!(path.exists());
}
