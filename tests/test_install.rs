use assert_cmd::Command;
use predicates::prelude::*;

fn bin() -> Command {
    Command::cargo_bin("jas").unwrap()
}

#[test]
fn test_install_gh() {
    let mut cmd = bin();
    let expected_url = "https://github.com/crate-ci/typos/releases/download/v1.31.1/";
    cmd.arg("--verbose")
        .arg("install")
        .arg("--gh")
        .arg("crate-ci/typos@v1.31.1")
        .arg("--dir")
        .arg("tests")
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
fn test_install_url() {
    let base = "https://www.johnvansickle.com/ffmpeg/old-releases/";
    let url = if cfg!(target_os = "macos") && cfg!(target_arch = "aarch64") {
        format!("{base}/ffmpeg-6.0.1-arm64-static.tar.xz")
    } else if cfg!(target_os = "linux") && cfg!(target_arch = "x86_64") {
        format!("{base}/ffmpeg-6.0.1-amd64-static.tar.xz")
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
    let path = std::path::Path::new("tests/ffmpeg");
    assert!(path.exists());

    let mut version_cmd = Command::new(path);
    version_cmd.arg("--version");
    version_cmd
        .assert()
        .success()
        .stdout(predicate::str::contains("6.0.1"));
}
