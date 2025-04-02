use crate::InstallArgs;
use bytes::Bytes;
use reqwest::get;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

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

async fn get_gh_asset_url(owner: &str, repo: &str, tag: &str) -> String {
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
    asset["browser_download_url"].as_str().unwrap().to_string()
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
        let actual = crate::sha::Sha256Hash::from_data(&body);
        if expected != &actual {
            panic!("SHA-256 mismatch: expected {expected}, got {actual}");
        }
    }
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
    let url = get_gh_asset_url(owner, repo, tag).await;
    tracing::info!("Downloading {}", url);
    let response = get(url).await.unwrap();
    let body = response.bytes().await.unwrap();
    verify_sha(&body, args);
    let dir = interpret_path(&args.dir);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(repo);
    let mut file = File::create(path).unwrap();
    file.write_all(&body).unwrap();
}

/// Install a binary.
pub(crate) async fn install(args: &InstallArgs) {
    if let Some(gh) = &args.gh {
        install_gh(gh, args).await;
    } else {
        todo!()
    }
}
