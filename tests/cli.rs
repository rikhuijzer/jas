//! Offline end-to-end tests of the standalone Bash CLI. Rust is only a test tool.
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use tempfile::TempDir;

fn git_bash_from_exec_path(exec_path: &Path) -> Option<PathBuf> {
    // Git for Windows may be installed anywhere. Walk up from its helper
    // directory to find Bash in that same installation, never through PATH.
    for directory in exec_path.ancestors() {
        for relative in ["bin/bash.exe", "usr/bin/bash.exe"] {
            let candidate = directory.join(relative);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

fn bash() -> Command {
    if let Some(executable) = std::env::var_os("JAS_TEST_BASH") {
        return Command::new(executable);
    }
    if cfg!(windows) {
        // Resolving "bash" through Windows PATH can select the WSL launcher.
        let output = Command::new("git")
            .arg("--exec-path")
            .output()
            .expect("Install Git for Windows to run the tests");
        assert!(output.status.success(), "git --exec-path failed");
        let exec_path = String::from_utf8(output.stdout).expect("Invalid Git installation path");
        let executable = git_bash_from_exec_path(Path::new(exec_path.trim()))
            .expect("Git Bash not found; install Git for Windows or set JAS_TEST_BASH");
        Command::new(executable)
    } else {
        Command::new("bash")
    }
}

struct Fixture {
    root: TempDir,
    script: PathBuf,
}
impl Fixture {
    fn new() -> Self {
        let root = tempfile::tempdir().unwrap();
        let script = Path::new(env!("CARGO_MANIFEST_DIR")).join("jas");
        let mock = root.path().join("mock");
        fs::create_dir(&mock).unwrap();
        fs::write(
            mock.join("curl"),
            r#"#!/usr/bin/env bash
set -euo pipefail
out=''
for ((i=1; i<=$#; i++)); do
    arg=${!i}
    if [[ "$arg" == --output || "$arg" == -o ]]; then
        j=$((i+1)); out=${!j}
    fi
done
printf '%s\n' "$@" >> "$JAS_REQUEST_LOG"
if [[ "${!#}" == https://api.github.com/* ]]; then
    cp "$JAS_RELEASE" "$out"
else
    [[ "${JAS_DOWNLOAD_FAIL:-false}" == false ]] || exit 22
    cp "$JAS_PAYLOAD" "$out"
fi
"#,
        )
        .unwrap();
        fs::write(
            mock.join("uname"),
            r#"#!/usr/bin/env bash
if [[ "$1" == -s && -n "${JAS_UNAME_OS:-}" ]]; then
    printf '%s\n' "$JAS_UNAME_OS"
else
    exec "$JAS_REAL_UNAME" "$@"
fi
"#,
        )
        .unwrap();
        fs::write(
            mock.join("jq"),
            r#"#!/usr/bin/env bash
set -euo pipefail
if [[ "${JAS_JQ_CRLF:-false}" == true ]]; then
    "$JAS_REAL_JQ" "$@" | tr -d '\r' | sed 's/$/\r/'
else
    exec "$JAS_REAL_JQ" "$@"
fi
"#,
        )
        .unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            for name in ["curl", "uname", "jq"] {
                fs::set_permissions(mock.join(name), fs::Permissions::from_mode(0o755)).unwrap();
            }
        }
        fs::write(root.path().join("payload"), b"hello world").unwrap();
        fs::write(root.path().join("github-path"), "").unwrap();
        Self { root, script }
    }
    fn path(&self, name: &str) -> PathBuf {
        self.root.path().join(name)
    }
    fn run_with(&self, args: &[&str], extra: &[(&str, &str)]) -> Output {
        let mut cmd = bash();
        // Assemble POSIX PATH inside Bash, also on Windows. The wrapper is
        // test-only; the executable itself has no source mode or test hooks.
        cmd.args([
            "-c",
            r#"set -e
if command -v cygpath >/dev/null 2>&1; then JAS_MOCK=$(cygpath -u "$JAS_MOCK"); fi
export JAS_REAL_UNAME=$(command -v uname)
export JAS_REAL_JQ=$(command -v jq || true)
export PATH="$JAS_MOCK:${JAS_TEST_PATH:-$PATH}"
exec "$BASH" "$@"
"#,
            "test",
        ])
        .arg(&self.script)
        .args(args)
        .env("JAS_MOCK", self.path("mock"))
        .env("JAS_PAYLOAD", self.path("payload"))
        .env("JAS_RELEASE", self.path("release.json"))
        .env("JAS_REQUEST_LOG", self.path("requests"))
        .env("GITHUB_PATH", self.path("github-path"))
        .env_remove("GITHUB_TOKEN")
        .env_remove("RUNNER_ARCH");
        for (key, value) in extra {
            cmd.env(key, value);
        }
        cmd.output().unwrap()
    }
    fn run(&self, args: &[&str]) -> Output {
        self.run_with(args, &[])
    }
    fn install(&self, args: &[&str]) -> Output {
        let dir = self.path("installed");
        let mut all = vec!["install", "--dir", dir.to_str().unwrap()];
        all.extend_from_slice(args);
        self.run(&all)
    }
    fn tar(&self, names: &[&str], gzip: bool) {
        let file = fs::File::create(self.path("payload")).unwrap();
        let writer: Box<dyn Write> = if gzip {
            Box::new(flate2::write::GzEncoder::new(
                file,
                flate2::Compression::default(),
            ))
        } else {
            Box::new(file)
        };
        let mut archive = tar::Builder::new(writer);
        for name in names {
            let mut header = tar::Header::new_gnu();
            header.set_size(11);
            header.set_mode(0o755);
            header.set_cksum();
            archive
                .append_data(&mut header, name, &b"hello world"[..])
                .unwrap();
        }
        archive.into_inner().unwrap().flush().unwrap();
    }
    fn zip(&self, names: &[&str]) {
        let mut zip = zip::ZipWriter::new(fs::File::create(self.path("payload")).unwrap());
        for name in names {
            zip.start_file(*name, zip::write::SimpleFileOptions::default())
                .unwrap();
            zip.write_all(b"hello world").unwrap();
        }
        zip.finish().unwrap();
    }
}
fn success(out: &Output) -> String {
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout.clone()).unwrap()
}
fn failure(out: &Output, message: &str) {
    assert!(!out.status.success());
    assert!(
        String::from_utf8_lossy(&out.stderr).contains(message),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}
const HELLO_SHA: &str = "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9";
#[test]
fn cli_information() {
    let f = Fixture::new();
    assert!(success(&f.run(&["--help"])).contains("--archive-filename"));
    assert_eq!(success(&f.run(&["--version"])), "jas 0.4.0\n");
    assert_eq!(
        success(&f.run(&["license"])).replace("\r\n", "\n"),
        include_str!("../LICENSE")
    );
    assert!(success(&f.run(&["show", "--install-dir"]))
        .trim()
        .ends_with("/.jas/bin"));
}
#[test]
fn hash_path_and_url() {
    let f = Fixture::new();
    assert_eq!(
        success(&f.run(&[
            "--verbose",
            "--ansi=false",
            "sha",
            "-p",
            f.path("payload").to_str().unwrap()
        ]))
        .trim(),
        HELLO_SHA
    );
    assert_eq!(
        success(&f.run(&["sha", "--url=example.com/tool"])).trim(),
        HELLO_SHA
    );
    assert!(fs::read_to_string(f.path("requests"))
        .unwrap()
        .contains("https://example.com/tool"));
}
/// The key security contract: an incorrect checksum must never install anything.
#[test]
fn incorrect_sha_does_not_touch_existing_installation() {
    let f = Fixture::new();
    fs::create_dir(f.path("installed")).unwrap();
    fs::write(f.path("installed/tool"), "old").unwrap();
    failure(
        &f.install(&["--url=https://example.com/tool", "--sha", &"0".repeat(64)]),
        "SHA-256 mismatch: expected",
    );
    assert_eq!(fs::read_to_string(f.path("installed/tool")).unwrap(), "old");
    // Even a corrupt archive is not inspected before its checksum is checked.
    failure(
        &f.install(&[
            "--url=https://example.com/tool.tar.gz",
            "--sha",
            &"0".repeat(64),
        ]),
        "SHA-256 mismatch",
    );
}
#[test]
fn raw_download_and_output_override() {
    let f = Fixture::new();
    success(&f.install(&[
        "--url",
        "https://example.com/tool-x86_64",
        "--sha",
        &HELLO_SHA.to_uppercase(),
        "--executable-filename",
        "custom.sh",
    ]));
    assert_eq!(
        fs::read(f.path("installed/custom.sh")).unwrap(),
        b"hello world"
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            fs::metadata(f.path("installed/custom.sh"))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o755
        );
    }
    assert!(fs::read_to_string(f.path("github-path"))
        .unwrap()
        .contains("installed"));
}
#[test]
fn nested_archive_and_multiple_renames() {
    let f = Fixture::new();
    f.tar(
        &[
            "package/bin/first.sh",
            "package/bin/second.sh",
            "package/README",
        ],
        true,
    );
    success(&f.install(&[
        "--url",
        "https://example.com/tools.tar.gz",
        "--archive-filename=first.sh",
        "--archive-filename",
        "second.sh",
        "--executable-filename=one.sh",
        "--executable-filename=two.sh",
    ]));
    assert!(f.path("installed/one.sh").is_file());
    assert!(f.path("installed/two.sh").is_file());
    assert!(!f.path("installed/README").exists());
}
#[test]
fn zip_guesses_exact_binary_before_similar_files() {
    let f = Fixture::new();
    f.zip(&["tool.sh", "tool.sh.1", "LICENSE"]);
    success(&f.install(&["--url=https://example.com/tool.sh-release.zip"]));
    assert_eq!(
        fs::read(f.path("installed/tool.sh")).unwrap(),
        b"hello world"
    );
}
#[test]
fn xz_archive() {
    let f = Fixture::new();
    fs::create_dir(f.path("nested")).unwrap();
    fs::write(f.path("nested/tool.sh"), "hello world").unwrap();
    let out = bash()
        .args([
            "-c",
            r#"set -e
archive=$1
root=$2
if command -v cygpath >/dev/null 2>&1; then
    archive=$(cygpath -u "$archive")
    root=$(cygpath -u "$root")
fi
tar -cJf "$archive" -C "$root" nested
"#,
            "test",
        ])
        .arg(f.path("payload"))
        .arg(f.root.path())
        .output()
        .unwrap();
    success(&out);
    let dir = f.path("installed");
    success(&f.run_with(
        &[
            "install",
            "--url=https://example.com/tool.sh-release.tar.xz",
            "--dir",
            dir.to_str().unwrap(),
        ],
        &[("TMPDIR", f.root.path().to_str().unwrap())],
    ));
    assert!(f.path("installed/tool.sh").is_file());
}
#[test]
fn invalid_arguments_fail_before_download() {
    let f = Fixture::new();
    for (args, message) in [
        (vec!["install"], "Expected either"),
        (vec!["install", "--url"], "Missing value"),
        (vec!["install", "--url=x", "--sha=xyz"], "64 hexadecimal"),
        (
            vec![
                "install",
                "--url=x",
                "--archive-filename=a",
                "--executable-filename=b",
                "--executable-filename=c",
            ],
            "counts must match",
        ),
        (
            vec!["install", "--url=x", "--executable-filename=../escape"],
            "basename",
        ),
        (vec!["sha", "--url=x", "--path=y"], "only one"),
        (vec!["install", "--gh=owner/repo"], "OWNER/REPO@TAG"),
        (vec!["--unknown"], "Unknown argument"),
        (vec!["--ansi=maybe", "show"], "true or false"),
    ] {
        failure(&f.run(&args), message);
    }
    assert!(!f.path("requests").exists());
}
#[test]
fn corrupt_archive_fails_without_creating_install_dir() {
    let f = Fixture::new();
    failure(
        &f.install(&["--url=https://example.com/tool.tar.gz"]),
        "Cannot list tar archive",
    );
    failure(
        &f.install(&["--url=https://example.com/tool.zip"]),
        "Cannot list ZIP archive",
    );
    assert!(!f.path("installed").exists());
}
#[test]
fn unsafe_zip_members_are_rejected() {
    let f = Fixture::new();
    for name in ["../escape", "/absolute", "C:/escape", "dir\\escape"] {
        f.zip(&[name]);
        failure(
            &f.install(&["--url=https://example.com/tool.zip"]),
            "Unsafe archive path",
        );
        assert!(!f.path("installed").exists());
    }
}
#[test]
fn tar_links_are_rejected() {
    let f = Fixture::new();
    let mut archive = tar::Builder::new(fs::File::create(f.path("payload")).unwrap());
    let mut h = tar::Header::new_gnu();
    h.set_entry_type(tar::EntryType::Symlink);
    h.set_size(0);
    h.set_mode(0o777);
    archive.append_link(&mut h, "tool", "../escape").unwrap();
    archive.finish().unwrap();
    drop(archive);
    failure(
        &f.install(&["--url=https://example.com/tool.tar.gz"]),
        "Archive links",
    );
}
#[test]
fn missing_archive_member_does_not_install_other_members() {
    let f = Fixture::new();
    f.zip(&["first.sh"]);
    failure(
        &f.install(&[
            "--url=https://example.com/tool.zip",
            "--archive-filename=first.sh",
            "--archive-filename=missing.sh",
        ]),
        "Could not find executable",
    );
    assert!(!f.path("installed").exists());
}
#[test]
fn http_failure_is_fatal() {
    let f = Fixture::new();
    let dir = f.path("installed");
    let out = f.run_with(
        &[
            "install",
            "--url=https://example.com/tool",
            "--dir",
            dir.to_str().unwrap(),
        ],
        &[("JAS_DOWNLOAD_FAIL", "true")],
    );
    assert_eq!(out.status.code(), Some(22));
    assert!(f.path("requests").is_file());
    assert!(!dir.exists());
}
#[test]
fn github_asset_selection_and_token() {
    let f = Fixture::new();
    let mut assets = vec![];
    for os in ["linux", "darwin", "windows"] {
        for arch in ["x86_64", "aarch64"] {
            for ext in [".sha256", ".pkg", ""] {
                let name = format!("tool-{arch}-{os}{ext}");
                assets.push(serde_json::json!({"name":name,"browser_download_url":format!("https://example.com/{name}")}));
            }
        }
    }
    fs::write(
        f.path("release.json"),
        serde_json::json!({"assets":assets}).to_string(),
    )
    .unwrap();
    // Use the real host OS, and exercise both architectures independently.
    let os = if cfg!(target_os = "macos") {
        "darwin"
    } else if cfg!(windows) {
        "windows"
    } else {
        "linux"
    };
    for (runner_arch, arch) in [("X64", "x86_64"), ("ARM64", "aarch64")] {
        let dir = f.path("installed");
        success(&f.run_with(
            &[
                "install",
                "--gh=owner/tool@release/v1",
                "--dir",
                dir.to_str().unwrap(),
            ],
            &[
                ("RUNNER_ARCH", runner_arch),
                ("GITHUB_TOKEN", "test-token"),
                ("JAS_JQ_CRLF", "true"),
            ],
        ));
        let requests = fs::read_to_string(f.path("requests")).unwrap();
        assert!(requests.contains(&format!("https://example.com/tool-{arch}-{os}\n")));
        assert!(requests.contains("release%2Fv1\n"));
        assert!(!requests.contains('\r'));
        assert!(requests.contains("Authorization: Bearer test-token"));
    }
    let dir = f.path("installed");
    success(&f.run_with(
        &[
            "install",
            "--dir",
            dir.to_str().unwrap(),
            "--gh=owner/tool@v1",
            "--asset-name=tool-aarch64-linux",
            "--gh-token=override",
        ],
        &[("JAS_JQ_CRLF", "true")],
    ));
    assert!(fs::read_to_string(f.path("requests"))
        .unwrap()
        .contains("Bearer override"));
    failure(
        &f.install(&["--gh=owner/tool@v1", "--asset-name=missing"]),
        "Asset not found",
    );
}

#[test]
fn windows_extension_rules() {
    let f = Fixture::new();
    f.zip(&["tool.exe"]);
    let dir = f.path("installed");
    success(&f.run_with(
        &[
            "install",
            "--url=https://example.com/tool-release.zip",
            "--dir",
            dir.to_str().unwrap(),
            "--archive-filename=tool",
            "--executable-filename=custom",
        ],
        &[("JAS_UNAME_OS", "MINGW64_NT-10.0")],
    ));
    assert!(f.path("installed/custom.exe").is_file());
    success(&f.run_with(
        &[
            "install",
            "--url=https://example.com/script",
            "--dir",
            dir.to_str().unwrap(),
            "--executable-filename=custom.py",
        ],
        &[("JAS_UNAME_OS", "MINGW64_NT-10.0")],
    ));
    assert!(f.path("installed/custom.py").is_file());
}

#[test]
fn zip_links_are_rejected() {
    let f = Fixture::new();
    let mut zip = zip::ZipWriter::new(fs::File::create(f.path("payload")).unwrap());
    zip.add_symlink(
        "tool",
        "../escape",
        zip::write::SimpleFileOptions::default(),
    )
    .unwrap();
    zip.finish().unwrap();
    failure(
        &f.install(&["--url=https://example.com/tool.zip"]),
        "Archive links",
    );
}

#[test]
#[cfg(unix)]
fn zip_with_bsdtar_and_no_unzip() {
    use std::os::unix::fs::symlink;
    let version = Command::new("tar").arg("--version").output().unwrap();
    if !String::from_utf8_lossy(&version.stdout).contains("bsdtar") {
        return;
    }
    let f = Fixture::new();
    let tools = f.path("tools");
    fs::create_dir(&tools).unwrap();
    for name in [
        "bash", "tar", "grep", "cp", "chmod", "mkdir", "mktemp", "rm", "uname",
    ] {
        let path = bash()
            .args(["-c", "command -v \"$1\"", "test", name])
            .output()
            .unwrap();
        symlink(success(&path).trim(), tools.join(name)).unwrap();
    }
    let dir = f.path("installed");
    for names in [vec!["tool.sh"], vec!["../escape"]] {
        f.zip(&names);
        let out = f.run_with(
            &[
                "install",
                "--url=https://example.com/tool.sh-release.zip",
                "--dir",
                dir.to_str().unwrap(),
            ],
            &[("JAS_TEST_PATH", tools.to_str().unwrap())],
        );
        if names[0] == "tool.sh" {
            success(&out);
            assert!(f.path("installed/tool.sh").is_file());
        } else {
            failure(&out, "Unsafe archive path");
        }
    }
}

#[test]
fn temporary_files_are_removed_on_success_and_failure() {
    let f = Fixture::new();
    let scratch = f.path("scratch");
    fs::create_dir(&scratch).unwrap();
    let dir = f.path("installed");
    for checksum in [HELLO_SHA, &"0".repeat(64)] {
        let out = f.run_with(
            &[
                "install",
                "--url=https://example.com/tool.sh",
                "--sha",
                checksum,
                "--dir",
                dir.to_str().unwrap(),
            ],
            &[("TMPDIR", scratch.to_str().unwrap())],
        );
        if checksum == HELLO_SHA {
            success(&out);
        } else {
            failure(&out, "SHA-256 mismatch");
        }
        assert_eq!(fs::read_dir(&scratch).unwrap().count(), 0);
    }
}

#[test]
fn finds_git_bash_in_a_custom_installation() {
    let root = tempfile::tempdir().unwrap();
    let install = root.path().join("Custom Git Installation");
    let exec_path = install.join("mingw64/libexec/git-core");
    fs::create_dir_all(&exec_path).unwrap();
    // A launcher elsewhere must not be selected, even if no Git Bash exists.
    fs::write(root.path().join("bash.exe"), "WSL launcher").unwrap();
    assert!(git_bash_from_exec_path(&exec_path).is_none());
    let bash_path = install.join("usr/bin/bash.exe");
    fs::create_dir_all(bash_path.parent().unwrap()).unwrap();
    fs::write(&bash_path, "Git Bash").unwrap();
    assert_eq!(git_bash_from_exec_path(&exec_path), Some(bash_path));
    // Git's bin launcher is supported too, including portable installations.
    let launcher = install.join("bin/bash.exe");
    fs::create_dir_all(launcher.parent().unwrap()).unwrap();
    fs::write(&launcher, "Git Bash launcher").unwrap();
    assert_eq!(git_bash_from_exec_path(&exec_path), Some(launcher));
}
