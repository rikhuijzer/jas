use crate::InstallArgs;
use reqwest::get;

async fn get_gh_release(owner: &str, repo: &str, tag: &str) -> String {
    let url = format!("https://api.github.com/repos/{owner}/{repo}/releases/tags/{tag}");
    let client = reqwest::Client::new();
    let response = client.get(url)
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .unwrap();
    let body = response.json::<serde_json::Value>().await.unwrap();

    todo!()
}

async fn install_gh(gh: &str, args: &InstallArgs) {
    let (owner, mut repo) = gh.split_once('/').unwrap();
    let tag = if let Some((repo, tag)) = repo.split_once('@') {
        tag
    } else {
        todo!("Assumes tag")
    };
    let url = format!("https://github.com/{owner}/{repo}/releases/tag/{tag}");
    let response = get(url).await.unwrap();
    let body = response.json::<serde_json::Value>().await.unwrap();
    let 
}

/// Install a binary.
pub(crate) async fn install(args: &InstallArgs) {
    if let Some(gh) = &args.gh {
        install_gh(gh, args).await;
    } else {
        todo!()
    }
}
