use assert_cmd::Command;
use predicates::prelude::*;

fn bin() -> Command {
    Command::cargo_bin("jas").unwrap()
}

#[test]
fn test_install_github() {
    let mut cmd = bin();
    let expected_url = "https://github.com/crate-ci/typos/releases/download/v1.31.1/typos-v1.31.1-aarch64-apple-darwin.tar.gz";
    cmd.arg("install")
        .arg("--gh")
        .arg("crate-ci/typos@v1.31.1")
        .arg("--dir")
        .arg("tests")
        .arg("--verbose")
        .assert()
        .success()
        .stdout(predicate::str::contains(expected_url));
    let path = std::path::Path::new("tests/typos");
    assert!(path.exists());
}
