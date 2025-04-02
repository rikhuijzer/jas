use assert_cmd::Command;
use predicates::prelude::*;

fn bin() -> Command {
    Command::cargo_bin("jas").unwrap()
}

#[test]
fn test_install_gh() {
    let mut cmd = bin();
    let expected_url = "https://github.com/crate-ci/typos/releases/download/v1.31.1/typos-v1.31.1-aarch64-apple-darwin.tar.gz";
    cmd.arg("--verbose")
        .arg("install")
        .arg("--gh")
        .arg("crate-ci/typos@v1.31.1")
        .arg("--dir")
        .arg("tests")
        .assert()
        .success()
        .stdout(predicate::str::contains(expected_url));
    let path = std::path::Path::new("tests/typos");
    assert!(path.exists());

    let mut version_cmd = Command::new(path);
    version_cmd.arg("--version");
    version_cmd
        .assert()
        .success()
        .stdout(predicate::str::contains("1.31.1"));
}
