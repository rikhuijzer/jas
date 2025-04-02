use assert_cmd::Command;
use predicates::prelude::*;

fn bin() -> Command {
    Command::cargo_bin("jas").unwrap()
}

fn clean_tests_dir(prefix: &str) {
    let tests_dir = std::path::Path::new("tests");
    if tests_dir.exists() {
        for entry in std::fs::read_dir(tests_dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .starts_with(prefix)
            {
                if path.is_file() {
                    std::fs::remove_file(path).unwrap();
                } else if path.is_dir() {
                    std::fs::remove_dir_all(path).unwrap();
                }
            }
        }
    }
}

#[test]
fn test_install_gh() {
    clean_tests_dir("typos");

    let mut cmd = bin();
    let expected_url = "https://github.com/crate-ci/typos/releases/download/v1.31.1/";
    cmd.arg("--verbose")
        .arg("install")
        .arg("--gh=crate-ci/typos@v1.31.1")
        .arg("--dir=tests")
        .arg("--sha=a172195e1b1f1e011b3034913d1c87f0bbf0552a096b4ead0e3fa0620f4329cd")
        .assert()
        .success()
        .stdout(predicate::str::contains(expected_url))
        .stdout(predicate::str::contains(
            "you may need to add it to your PATH manually",
        ));
    let path = std::path::Path::new("tests/typos");
    assert!(path.exists());

    let mut version_cmd = Command::new(path);
    version_cmd.arg("--version");
    version_cmd
        .assert()
        .success()
        .stdout(predicate::str::contains("1.31.1"));
}

#[test]
fn test_install_gh_no_guesses() {
    clean_tests_dir("no_guess_typos");

    let mut cmd = bin();
    cmd.arg("--verbose")
        .arg("install")
        .arg("--gh=crate-ci/typos@v1.31.0")
        .arg("--archive-filename=this_file_does_not_exist")
        .arg("--binary-filename=no_guess_typos")
        .arg("--sha=96684058f88bd8343aa992223c9937f399254eb5277f0d297d2ac7b022d990b7")
        .arg("--dir=tests")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Could not find binary in archive; file this_file_does_not_exist not in",
        ));

    let mut cmd = bin();
    cmd.arg("--verbose")
        .arg("install")
        .arg("--gh=crate-ci/typos@v1.31.0")
        .arg("--archive-filename=typos")
        .arg("--binary-filename=no_guess_typos")
        .arg("--sha=96684058f88bd8343aa992223c9937f399254eb5277f0d297d2ac7b022d990b7")
        .arg("--dir=tests")
        .assert()
        .success();

    let path = std::path::Path::new("tests/no_guess_typos");
    assert!(path.exists());

    let mut version_cmd = Command::new(path);
    version_cmd.arg("--version");
    version_cmd
        .assert()
        .success()
        .stdout(predicate::str::contains("1.31.0"));
}

#[test]
fn test_install_url() {
    clean_tests_dir("trv");

    let base = "https://github.com/transformrs/trv/releases/download/v0.5.0/";
    let url = if cfg!(target_os = "macos") && cfg!(target_arch = "aarch64") {
        format!("{base}/trv-aarch64-apple-darwin")
    } else if cfg!(target_os = "linux") && cfg!(target_arch = "x86_64") {
        format!("{base}/trv-x86_64-unknown-linux-gnu")
    } else {
        return;
    };
    let mut cmd = bin();
    cmd.arg("--verbose")
        .arg("install")
        .arg("--url")
        .arg(&url)
        .arg("--dir")
        .arg("tests")
        .assert()
        .success()
        .stdout(predicate::str::contains(url))
        .stdout(predicate::str::contains(
            "you may need to add it to your PATH manually",
        ));
    let path = std::path::Path::new("tests/trv");
    assert!(path.exists());

    let mut version_cmd = Command::new(path);
    version_cmd.arg("--version");
    version_cmd
        .assert()
        .success()
        .stdout(predicate::str::contains("0.5.0"));
}
