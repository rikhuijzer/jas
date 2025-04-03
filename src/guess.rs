use crate::abort;
use std::path::PathBuf;

#[derive(Debug, PartialEq)]
enum TargetOs {
    Linux,
    Macos,
    Windows,
}

#[derive(Debug, PartialEq)]
enum TargetArch {
    X86_64,
    Aarch64,
    Arm,
}

fn contains_x86_64(name: &str) -> bool {
    name.contains("x86_64") || name.contains("amd64")
}

fn guess_asset_enum(names: &[&str], target_os: TargetOs, target_arch: TargetArch) -> usize {
    let searcher = |name: &&str| {
        if target_os == TargetOs::Linux && target_arch == TargetArch::X86_64 {
            name.contains("linux") && contains_x86_64(name)
        } else if target_os == TargetOs::Macos && target_arch == TargetArch::X86_64 {
            (name.contains("macos") || name.contains("darwin")) && contains_x86_64(name)
        } else if target_os == TargetOs::Macos && target_arch == TargetArch::Aarch64 {
            (name.contains("macos") || name.contains("darwin")) && name.contains("aarch64")
        } else if target_os == TargetOs::Linux && target_arch == TargetArch::Aarch64 {
            name.contains("linux") && name.contains("aarch64")
        } else if target_os == TargetOs::Linux && target_arch == TargetArch::Arm {
            name.contains("linux") && name.contains("arm")
        } else if target_os == TargetOs::Windows && target_arch == TargetArch::X86_64 {
            name.contains("windows") && contains_x86_64(name)
        } else {
            tracing::error!("Unsupported platform: {}", name);
            std::process::exit(1);
        }
    };
    names.iter().position(searcher).expect("No asset found")
}

/// Guess the asset name for the current platform.
pub fn guess_asset(names: &[&str]) -> usize {
    let target_os = if cfg!(target_os = "linux") {
        TargetOs::Linux
    } else if cfg!(target_os = "macos") {
        TargetOs::Macos
    } else if cfg!(target_os = "windows") {
        TargetOs::Windows
    } else {
        tracing::error!("Unsupported platform");
        std::process::exit(1);
    };
    let target_arch = if cfg!(target_arch = "x86_64") {
        TargetArch::X86_64
    } else if cfg!(target_arch = "aarch64") {
        TargetArch::Aarch64
    } else if cfg!(target_arch = "arm") {
        TargetArch::Arm
    } else {
        tracing::error!("Unsupported architecture");
        std::process::exit(1);
    };
    guess_asset_enum(names, target_os, target_arch)
}

#[test]
fn test_guess_asset_typos() {
    let names = vec![
        "typos-v1.31.1-aarch64-apple-darwin.tar.gz",
        "typos-v1.31.1-aarch64-unknown-linux-musl.tar.gz",
        "typos-v1.31.1-x86_64-apple-darwin.tar.gz",
        "typos-v1.31.1-x86_64-pc-windows-msvc.zip",
        "typos-v1.31.1-x86_64-unknown-linux-musl.tar.gz",
    ];
    let index = guess_asset_enum(&names, TargetOs::Macos, TargetArch::Aarch64);
    assert_eq!(index, 0);
    let index = guess_asset_enum(&names, TargetOs::Linux, TargetArch::X86_64);
    assert_eq!(index, 4);
    let index = guess_asset_enum(&names, TargetOs::Windows, TargetArch::X86_64);
    assert_eq!(index, 3);
}

#[test]
fn test_guess_asset_pandoc() {
    let names = vec![
        "pandoc-3.6.4-1-amd64.deb",
        "pandoc-3.6.4-linux-amd64.tar.gz",
        "pandoc-3.6.4-linux-arm64.tar.gz",
    ];
    let index = guess_asset_enum(&names, TargetOs::Linux, TargetArch::X86_64);
    assert_eq!(index, 1);
    let index = guess_asset_enum(&names, TargetOs::Linux, TargetArch::Arm);
    assert_eq!(index, 2);
}

pub(crate) fn guess_binary_in_archive(files: &[PathBuf], name: &str) -> PathBuf {
    let file = files
        .iter()
        .filter_map(|file| {
            let path = file;
            if let Some(current) = path.file_name() {
                let current = current.to_str().unwrap();
                tracing::debug!("Checking {current} against {name}");
                if current == name {
                    return Some(path.clone());
                }
                if current.contains(name) && !current.contains("LICENSE") {
                    return Some(path.clone());
                }
                None
            } else {
                None
            }
        })
        .next();

    if let Some(file) = file {
        file
    } else {
        let files = files
            .iter()
            .map(|file| file.display().to_string())
            .collect::<Vec<_>>();
        abort(&format!(
            "Could not find binary in archive; specify a binary name with --archive-filename. Available files in archive:\n{}",
            files.join("\n")
        ));
    }
}

#[test]
fn test_guess_binary_filename() {
    let files = vec![
        "typos-v1.31.1-x86_64-apple-darwin.tar.gz",
        "doc",
        "LICENSE",
        "README.md",
    ];
    let files = files.iter().map(PathBuf::from).collect::<Vec<_>>();
    let archive_filename = "typos-v1.31.1-x86_64-apple-darwin.tar.gz";
    let binary = guess_binary_in_archive(&files, archive_filename);
    assert_eq!(
        binary,
        PathBuf::from("typos-v1.31.1-x86_64-apple-darwin.tar.gz")
    );
}

pub(crate) fn guess_binary_filename_from_url(url: &str) -> String {
    let name = url.split('/').last().unwrap();
    let name = name.split('-').next().unwrap();
    name.to_string()
}

#[test]
fn test_guess_binary_filename_from_url() {
    let url = "https://github.com/typos-ci/typos/releases/download/v1.31.1/typos-v1.31.1-x86_64-apple-darwin.tar.gz";
    let name = guess_binary_filename_from_url(url);
    assert_eq!(name, "typos");
}
