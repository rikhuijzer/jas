use crate::InstallArgs;
use reqwest::get;
use std::env::consts::OS;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

/// Guess the asset name for the current platform.
fn guess_asset(names: &[&str]) -> usize {
    fn searcher(name: &&str) -> bool {
        if cfg!(target_os = "linux") && cfg!(target_arch = "x86_64") {
            name.contains("linux") && name.contains("x86_64")
        } else if cfg!(target_os = "macos") && cfg!(target_arch = "x86_64") {
            name.contains("macos") && name.contains("x86_64")
        } else if cfg!(target_os = "macos") && cfg!(target_arch = "aarch64") {
            name.contains("macos") && name.contains("arm64")
        } else if cfg!(target_os = "linux") && cfg!(target_arch = "aarch64") {
            name.contains("linux") && name.contains("arm64")
        } else if cfg!(target_os = "windows") && cfg!(target_arch = "x86_64") {
            name.contains("windows") && name.contains("x86_64")
        } else {
            panic!("Unsupported platform: {}", name);
        }
    }
    names.iter().position(searcher).expect("No asset found")
}

async fn get_gh_asset_url(owner: &str, repo: &str, tag: &str) -> String {
    let url = format!("https://api.github.com/repos/{owner}/{repo}/releases/tags/{tag}");
    let client = reqwest::Client::new();
    let response = client.get(url)
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .unwrap();
    let body = response.json::<serde_json::Value>().await.unwrap();
    let assets = body["assets"].as_array().unwrap();
    let names = assets.iter().map(|asset| asset["name"].as_str().unwrap()).collect::<Vec<_>>();
    let index = guess_asset(&names);
    let asset = &assets[index];
    asset["browser_download_url"].as_str().unwrap().to_string()
}

fn interpret_path(path: &str) -> PathBuf {
    if path.starts_with("~/") {
        PathBuf::from(std::env::var("HOME").unwrap()).join(&path[2..])
    } else {
        PathBuf::from(path)
    }
}

async fn install_gh(gh: &str, args: &InstallArgs) {
    let (owner, mut repo) = gh.split_once('/').unwrap();
    let tag = if let Some((repo, tag)) = repo.split_once('@') {
        tag
    } else {
        todo!("Assumes tag")
    };
    let url = get_gh_asset_url(owner, repo, tag).await;
    let response = get(url).await.unwrap();
    let body = response.bytes().await.unwrap();
    let dir = interpret_path(&args.dir);
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
