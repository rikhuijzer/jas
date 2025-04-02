use assert_cmd::Command;
use predicates::prelude::*;

fn bin() -> Command {
    Command::cargo_bin("jas").unwrap()
}

#[test]
fn test_sha() {
    // Obtained via `sha256sum LICENSE`.
    let expected = "399e6f883b8d97f822e8b9662d5377820d46f60dd33e95881e3173cebea6d70c";
    let mut cmd = bin();
    cmd.arg("sha")
        .arg("--path")
        .arg("LICENSE")
        .assert()
        .success()
        .stdout(predicate::str::contains(expected));
}

// #[test]
// fn test_sha_github() {
//     let expected = "399e6f883b8d97f822e8b9662d5377820d46f60dd33e95881e3173cebea6d70c";
//     let mut cmd = bin();
//     cmd.arg("sha")
//         .arg("--gh")
//         .arg("crate-ci/typos@v1.31.1")
//         .assert()
//         .success()
//         .stdout(predicate::str::contains(expected));
// }
