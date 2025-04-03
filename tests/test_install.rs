use assert_cmd::Command;
use predicates::prelude::*;
use std::path::Path;

fn bin() -> Command {
    Command::cargo_bin("jas").unwrap()
}

fn add_exe_if_needed(path: &str) -> String {
    if cfg!(target_os = "windows") {
        if !path.ends_with(".exe") {
            return format!("{path}.exe");
        }
    }
    path.to_string()
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
fn test_install_gh_guess_typos() {
    clean_tests_dir("typos");

    let sha = if cfg!(target_os = "macos") && cfg!(target_arch = "aarch64") {
        "a172195e1b1f1e011b3034913d1c87f0bbf0552a096b4ead0e3fa0620f4329cd"
    } else if cfg!(target_os = "linux") && cfg!(target_arch = "x86_64") {
        "f683c2abeaff70379df7176110100e18150ecd17a4b9785c32908aca11929993"
    } else if cfg!(target_os = "windows") {
        "1a8b5a2f2f7aaf9d07ac9b4a2039b9ae38722e12fd4afd5a08d6bdc8435f4279"
    } else {
        tracing::warn!("Skipping test on this platform");
        return;
    };
    let mut cmd = bin();
    let expected_url = "https://github.com/crate-ci/typos/releases/download/v1.31.1/";
    cmd.arg("--verbose")
        .arg("--ansi=false")
        .arg("install")
        .arg("--gh=crate-ci/typos@v1.31.1")
        .arg("--dir=tests")
        .arg(format!("--sha={sha}"))
        .assert()
        .success()
        .stderr(predicate::str::contains(expected_url))
        .stderr(predicate::str::contains(
            "you may need to add it to your PATH manually",
        ));
    let path = add_exe_if_needed("tests/typos");
    let path = Path::new(&path);
    assert!(path.exists());

    let mut version_cmd = Command::new(path);
    version_cmd.arg("--version");
    version_cmd
        .assert()
        .success()
        .stdout(predicate::str::contains("1.31.1"));
}

#[test]
fn test_install_gh_guess_just() {
    clean_tests_dir("just");

    let sha = if cfg!(target_os = "macos") && cfg!(target_arch = "aarch64") {
        "0fb2401a46409bdf574f42f92df0418934166032ec2bcb0fc7919b7664fdcc01"
    } else if cfg!(target_os = "linux") && cfg!(target_arch = "x86_64") {
        "181b91d0ceebe8a57723fb648ed2ce1a44d849438ce2e658339df4f8db5f1263"
    } else {
        tracing::warn!("Skipping test on this platform");
        return;
    };
    let mut cmd = bin();
    let expected_url = "https://github.com/casey/just/releases/download/1.40.0/";
    cmd.arg("--verbose")
        .arg("--ansi=false")
        .arg("install")
        .arg("--gh=casey/just@1.40.0")
        .arg("--dir=tests")
        .arg(format!("--sha={sha}"))
        .assert()
        .success()
        .stderr(predicate::str::contains(expected_url))
        .stderr(predicate::str::contains(
            "you may need to add it to your PATH manually",
        ));
    let path = add_exe_if_needed("tests/just");
    let path = Path::new(&path);
    assert!(path.exists());

    let mut version_cmd = Command::new(path);
    version_cmd.arg("--version");
    version_cmd
        .assert()
        .success()
        .stdout(predicate::str::contains("1.40.0"));
}

#[test]
fn test_install_gh_no_guesses() {
    clean_tests_dir("no_guess_typos");

    let sha = if cfg!(target_os = "macos") && cfg!(target_arch = "aarch64") {
        "96684058f88bd8343aa992223c9937f399254eb5277f0d297d2ac7b022d990b7"
    } else if cfg!(target_os = "linux") && cfg!(target_arch = "x86_64") {
        "3b11f5e3de56ecdc13fedc9425f201c83bd2dd045df938a166d7fed85d238faf"
    } else {
        tracing::warn!("Skipping test on this platform");
        return;
    };
    let asset_name = if cfg!(target_os = "macos") && cfg!(target_arch = "aarch64") {
        "typos-v1.31.0-aarch64-apple-darwin.tar.gz"
    } else {
        "typos-v1.31.0-x86_64-unknown-linux-musl.tar.gz"
    };
    let mut cmd = bin();
    cmd.arg("--verbose")
        .arg("--ansi=false")
        .arg("install")
        .arg("--gh=crate-ci/typos@v1.31.0")
        .arg(format!("--asset-name={asset_name}"))
        .arg("--archive-filename=this_file_does_not_exist")
        .arg("--binary-filename=no_guess_typos")
        .arg(format!("--sha={sha}"))
        .arg("--dir=tests")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Could not find binary in archive; file this_file_does_not_exist not in",
        ));

    let mut cmd = bin();
    cmd.arg("--verbose")
        .arg("--ansi=false")
        .arg("install")
        .arg("--gh=crate-ci/typos@v1.31.0")
        .arg("--archive-filename=typos")
        .arg("--binary-filename=no_guess_typos")
        .arg(format!("--sha={sha}"))
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
        .arg("--ansi=false")
        .arg("install")
        .arg("--url")
        .arg(&url)
        .arg("--dir")
        .arg("tests")
        .assert()
        .success()
        .stderr(predicate::str::contains(url))
        .stderr(predicate::str::contains(
            "you may need to add it to your PATH manually",
        ));
    let path = add_exe_if_needed("tests/trv");
    let path = Path::new(&path);
    assert!(path.exists());

    let mut version_cmd = Command::new(path);
    version_cmd.arg("--version");
    version_cmd
        .assert()
        .success()
        .stdout(predicate::str::contains("0.5.0"));
}
