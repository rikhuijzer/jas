use crate::InstallArgs;
use bytes::Bytes;
use flate2::read::GzDecoder;
use reqwest::get;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::path::PathBuf;
use tar::Archive;

/// Guess the asset name for the current platform.
fn guess_asset(names: &[&str]) -> usize {
    fn searcher(name: &&str) -> bool {
        if cfg!(target_os = "linux") && cfg!(target_arch = "x86_64") {
            name.contains("linux") && name.contains("x86_64")
        } else if cfg!(target_os = "macos") && cfg!(target_arch = "x86_64") {
            (name.contains("macos") || name.contains("darwin")) && name.contains("x86_64")
        } else if cfg!(target_os = "macos") && cfg!(target_arch = "aarch64") {
            (name.contains("macos") || name.contains("darwin")) && name.contains("aarch64")
        } else if cfg!(target_os = "linux") && cfg!(target_arch = "aarch64") {
            name.contains("linux") && name.contains("aarch64")
        } else if cfg!(target_os = "windows") && cfg!(target_arch = "x86_64") {
            name.contains("windows") && name.contains("x86_64")
        } else {
            panic!("Unsupported platform: {}", name);
        }
    }
    names.iter().position(searcher).expect("No asset found")
}

fn user_agent() -> String {
    format!("jas/{}", env!("CARGO_PKG_VERSION"))
}

async fn get_gh_asset_info(owner: &str, repo: &str, tag: &str) -> (String, String) {
    let url = format!("https://api.github.com/repos/{owner}/{repo}/releases/tags/{tag}");
    tracing::debug!("Requesting asset list from {}", url);
    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .header("User-Agent", user_agent())
        .send()
        .await;
    let response = match response {
        Ok(response) => response,
        Err(e) => {
            panic!("Error requesting asset list: {}", e);
        }
    };
    let bytes = response.bytes().await.unwrap();
    let body = match serde_json::from_slice::<serde_json::Value>(&bytes) {
        Ok(body) => body,
        Err(e) => {
            panic!("Error parsing asset list: {e}\nGot: {bytes:?}");
        }
    };
    let assets = body["assets"].as_array().unwrap();
    let names = assets
        .iter()
        .map(|asset| asset["name"].as_str().unwrap())
        .collect::<Vec<_>>();
    let index = guess_asset(&names);
    let asset = &assets[index];
    let url = asset["browser_download_url"].as_str().unwrap().to_string();
    let name = asset["name"].as_str().unwrap().to_string();
    (url, name)
}

fn interpret_path(path: &str) -> PathBuf {
    if let Some(prefix) = path.strip_prefix("~/") {
        PathBuf::from(std::env::var("HOME").unwrap()).join(prefix)
    } else {
        PathBuf::from(path)
    }
}

fn verify_sha(body: &Bytes, args: &InstallArgs) {
    if let Some(expected) = &args.sha {
        let actual = crate::sha::Sha256Hash::from_data(body);
        if expected != &actual {
            panic!("SHA-256 mismatch: expected {expected}, got {actual}");
        }
    }
}

/// Unpack a gzipped archive into a directory.
fn unpack_gz(body: &Bytes, dir: &Path, name: &str) -> Option<PathBuf> {
    if name.ends_with(".tar.gz") {
        let archive_dir = dir.join(name.strip_suffix(".tar.gz").unwrap());
        if archive_dir.exists() {
            if archive_dir.is_dir() {
                std::fs::remove_dir_all(&archive_dir).unwrap();
            } else {
                std::fs::remove_file(&archive_dir).unwrap();
            }
        }
        std::fs::create_dir_all(&archive_dir).unwrap();
        let decompressed = GzDecoder::new(body.as_ref());
        let mut archive = Archive::new(decompressed);
        archive.unpack(&archive_dir).unwrap();
        Some(archive_dir)
    } else {
        None
    }
}

fn verify_in_path(dir: &Path) {
    tracing::debug!("Verifying whether {dir:?} is in PATH");
    let path = std::env::var("PATH").unwrap();
    let paths = path.split(':').collect::<Vec<_>>();
    let mut found = false;
    for p in paths {
        let p = PathBuf::from(p);
        if p.exists() && p == dir {
            tracing::debug!("Found {dir:?} in PATH");
            found = true;
            break;
        }
    }
    if !found {
        tracing::warn!(
            "Could not find {dir:?} in PATH, you may need to add it to your PATH manually"
        );
    }
}

/// Copy the binary from the archive to the target directory.
fn copy_from_archive(dir: &Path, archive_dir: &PathBuf, name: &str) -> PathBuf {
    let files = std::fs::read_dir(archive_dir).unwrap();
    let binary = files
        .filter_map(|file| {
            let file = match file {
                Ok(file) => file,
                Err(_) => return None,
            };
            let path = file.path();
            if path.is_file() {
                if let Some(current) = path.file_name() {
                    let current = current.to_str().unwrap();
                    tracing::debug!("Checking {current} against {name}");
                    if name.contains(current) && !name.contains("LICENSE") {
                        Some(path)
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        })
        .next();
    if let Some(binary) = binary {
        let name = binary.file_name().unwrap();
        let mut src = File::open(&binary).unwrap();
        let dst_dir = dir.join(name);
        let mut dst = File::create(&dst_dir).unwrap();
        std::io::copy(&mut src, &mut dst).unwrap();
        let dst = dst_dir.display();
        tracing::info!("Placed binary at {dst}");
        dst_dir
    } else {
        panic!("Could not find binary in archive");
    }
}

fn make_executable(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    let mut permissions = std::fs::metadata(path).unwrap().permissions();
    permissions.set_mode(0o755);
    match std::fs::set_permissions(path, permissions) {
        Ok(_) => (),
        Err(e) => {
            tracing::warn!("Failed to set executable permissions: {e}\nPlease set the executable permissions manually:\nchmod +x {}", path.display());
        }
    }
}

async fn install_core(url: &str, args: &InstallArgs, filename: &str, output_name: &str) {
    tracing::info!("Downloading {}", url);
    let response = get(url).await.unwrap();
    let body = response.bytes().await.unwrap();
    verify_sha(&body, args);
    let dir = interpret_path(&args.dir);
    std::fs::create_dir_all(&dir).unwrap();
    let archive_dir = unpack_gz(&body, &dir, filename);
    if let Some(archive_dir) = archive_dir {
        let path = copy_from_archive(&dir, &archive_dir, filename);
        make_executable(&path);
    } else {
        let path = dir.join(output_name);
        let mut file = File::create(&path).unwrap();
        make_executable(&path);
        file.write_all(&body).unwrap();
    }
    verify_in_path(&dir);
}

async fn install_gh(gh: &str, args: &InstallArgs) {
    let split = gh.split_once('/').unwrap();
    let owner = split.0;
    let mut repo = split.1;
    let tag = if let Some((repo_, tag)) = repo.split_once('@') {
        repo = repo_;
        tag
    } else {
        todo!("Missing tag not yet supported")
    };
    let (url, name) = get_gh_asset_info(owner, repo, tag).await;
    install_core(&url, args, &name, repo).await;
}

fn guess_name(url: &str) -> String {
    let name = url.split('/').last().unwrap();
    let name = name.split('-').next().unwrap();
    name.to_string()
}

async fn install_url(url: &str, args: &InstallArgs) {
    let filename = url.split('/').last().unwrap();
    let output_name = guess_name(url);
    install_core(url, args, filename, &output_name).await;
}

/// Install a binary.
pub(crate) async fn install(args: &InstallArgs) {
    if let Some(gh) = &args.gh {
        install_gh(gh, args).await;
    } else if let Some(url) = &args.url {
        install_url(url, args).await;
    } else {
        todo!()
    }
}
