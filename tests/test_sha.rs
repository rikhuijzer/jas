use assert_cmd::Command;
use predicates::prelude::*;

fn bin() -> Command {
    Command::cargo_bin("jas").unwrap()
}

#[test]
fn test_sha() {
    // Obtained via `sha256sum LICENSE`.
    let expected = "3066dd79d02e7449fa493a6ac730ffd63319451b85e528d162d9e4725b8e0982";
    let mut cmd = bin();
    cmd.arg("--verbose")
        .arg("--ansi=false")
        .arg("sha")
        .arg("--path")
        .arg("LICENSE")
        .assert()
        .success()
        .stdout(predicate::str::contains(expected));
}

#[test]
fn test_sha_url() {
    let url = "github.com/crate-ci/typos/releases/download/v1.31.1/typos-v1.31.1-x86_64-unknown-linux-musl.tar.gz";
    let expected = "f683c2abeaff70379df7176110100e18150ecd17a4b9785c32908aca11929993";
    let mut cmd = bin();
    cmd.arg("--verbose")
        .arg("--ansi=false")
        .arg("sha")
        .arg(format!("--url={url}"))
        .assert()
        .success()
        .stderr(predicate::str::contains(url))
        .stdout(predicate::str::contains(expected));
}
