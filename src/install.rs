use crate::abort;
use crate::guess::guess_asset;
use crate::InstallArgs;
use flate2::read::GzDecoder;
use serde_json::Value;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::path::PathBuf;
use tar::Archive;

fn user_agent() -> String {
    format!("jas/{}", env!("CARGO_PKG_VERSION"))
}

fn find_gh_asset(args: &InstallArgs, assets: &[Value]) -> Value {
    let names = assets
        .iter()
        .map(|asset| asset["name"].as_str().unwrap())
        .collect::<Vec<_>>();
    let index = if let Some(name) = &args.asset_name {
        names.iter().position(|current| current == name).unwrap()
    } else {
        guess_asset(&names)
    };
    let asset = &assets[index];
    asset.clone()
}

fn get_gh_asset_info(args: &InstallArgs, owner: &str, repo: &str, tag: &str) -> (String, String) {
    let url = format!("https://api.github.com/repos/{owner}/{repo}/releases/tags/{tag}");
    tracing::debug!("Requesting asset list from {}", url);
    let mut request = ureq::get(url)
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .header("User-Agent", user_agent());
    if let Ok(token) = std::env::var("GITHUB_TOKEN") {
        let token = format!("Bearer {token}");
        request = request.header("Authorization", token);
    }
    let response = request.call();
    let mut response = match response {
        Ok(response) => response,
        Err(e) => {
            abort(&format!(
                "Error requesting asset list. Is the correct tag specified? Error: {e}"
            ));
        }
    };
    let bytes = match response.body_mut().read_to_vec() {
        Ok(bytes) => bytes,
        Err(e) => {
            abort(&format!("Error reading asset list: {e}"));
        }
    };
    let body = match serde_json::from_slice::<serde_json::Value>(&bytes) {
        Ok(body) => body,
        Err(e) => {
            abort(&format!("Error parsing asset list: {e}\nGot: {bytes:?}"));
        }
    };
    let assets = match body["assets"].as_array() {
        Some(assets) => assets,
        None => {
            abort(&format!("Unexpected response from GitHub: {body}"));
        }
    };
    let asset = find_gh_asset(args, assets);
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

fn verify_sha(body: &[u8], args: &InstallArgs) {
    if let Some(expected) = &args.sha {
        let actual = crate::sha::Sha256Hash::from_data(body);
        if expected != &actual {
            abort(&format!(
                "SHA-256 mismatch: expected\n{expected}, but got\n{actual}"
            ));
        }
    }
}

/// Unpack a gzipped archive into a directory.
fn unpack_archive(body: &[u8], dir: &Path, name: &str) -> Option<PathBuf> {
    let stem = Path::new(name).file_stem();
    let archive_dir = dir.join(stem.as_ref().unwrap());
    if name.ends_with(".tar.gz") || name.ends_with(".zip") {
        if archive_dir.exists() {
            if archive_dir.is_dir() {
                std::fs::remove_dir_all(&archive_dir).unwrap();
            } else {
                std::fs::remove_file(&archive_dir).unwrap();
            }
        }
        std::fs::create_dir_all(&archive_dir).unwrap();
    }
    if name.ends_with(".tar.gz") {
        std::fs::create_dir_all(&archive_dir).unwrap();
        let decompressed = GzDecoder::new(body);
        let mut archive = Archive::new(decompressed);
        archive.unpack(&archive_dir).unwrap();
        tracing::debug!("Unpacked archive into {}", archive_dir.display());
        Some(archive_dir)
    } else if name.ends_with(".zip") {
        #[cfg(windows)]
        {
            use zip::unstable::stream::ZipStreamReader;
            let archive_dir = dir.join(name.strip_suffix(".zip").unwrap());
            let zip = ZipStreamReader::new(body.as_ref());
            zip.extract(&archive_dir).unwrap();
            tracing::debug!("Unpacked archive into {}", archive_dir.display());
            Some(archive_dir)
        }
        #[cfg(not(windows))]
        abort("Zip archives are (currently) only supported on Windows.");
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

fn add_exe_if_needed(path: &Path) -> PathBuf {
    if cfg!(target_os = "windows") {
        path.with_extension("exe")
    } else {
        path.to_path_buf()
    }
}

/// Return the files in an archive.
/// 
/// Also handles archives with nested directories.
fn files_in_archive(archive_dir: &Path) -> Vec<PathBuf> {
    let files = std::fs::read_dir(archive_dir).unwrap();
    let files = files.map(|file| file.unwrap().path()).collect::<Vec<_>>();
    if files.len() == 1 {
        let path = &files[0];
        // If the archive contains a single dir, read the files in that dir.
        if path.is_dir() {
            let files = files_in_archive(path);
            let dirname = PathBuf::from(path.file_name().unwrap());
            files
                .into_iter()
                .map(|file| {
                    let filename = file.file_name().unwrap();
                    archive_dir.join(&dirname).join(filename)
                })
                .collect()
        } else {
            files
        }
    } else {
        files
    }
}

/// Copy the binary from the archive to the target directory.
fn copy_from_archive(dir: &Path, archive_dir: &Path, args: &InstallArgs, name: &str) -> PathBuf {
    let binary = if let Some(filename) = &args.archive_filename {
        let filename = add_exe_if_needed(Path::new(filename));
        let binary = archive_dir.join(&filename);
        if binary.exists() {
            binary
        } else {
            let files = files_in_archive(archive_dir);
            let files = files
                .iter()
                .map(|file| file.display().to_string())
                .collect::<Vec<_>>();
            abort(&format!(
                "Could not find binary in archive; file {} not in\n{}",
                filename.display(),
                files.join("\n")
            ));
        }
    } else {
        let files = files_in_archive(archive_dir);
        let binary = crate::guess::guess_binary_in_archive(&files, name);
        add_exe_if_needed(&binary)
    };
    let filename = binary.file_name().unwrap();
    let mut src = File::open(&binary).unwrap();
    let dst_path = if let Some(filename) = &args.binary_filename {
        dir.join(filename)
    } else {
        dir.join(filename)
    };
    let mut dst = File::create(&dst_path).unwrap();
    std::io::copy(&mut src, &mut dst).unwrap();
    let dst = dst_path.display();
    tracing::info!("Placed binary at {dst}");
    dst_path
}

fn make_executable(path: &Path) {
    #[cfg(unix)]
    {
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
}

fn install_core(url: &str, args: &InstallArgs, name: &str, output_name: &str) {
    tracing::info!("Downloading {}", url);
    let mut response = match ureq::get(url).call() {
        Ok(response) => response,
        Err(e) => {
            abort(&format!("Error downloading {url}: {e}"));
        }
    };
    let body = match response.body_mut().read_to_vec() {
        Ok(body) => body,
        Err(e) => {
            abort(&format!("Error reading {url}: {e}"));
        }
    };
    verify_sha(&body, args);
    let dir = interpret_path(&args.dir);
    std::fs::create_dir_all(&dir).unwrap();
    let archive_dir = unpack_archive(&body, &dir, name);
    if let Some(archive_dir) = archive_dir {
        let path = copy_from_archive(&dir, &archive_dir, args, output_name);
        make_executable(&path);
    } else {
        let path = dir.join(output_name);
        let mut file = File::create(&path).unwrap();
        make_executable(&path);
        file.write_all(&body).unwrap();
    }
    verify_in_path(&dir);
}

fn install_gh(gh: &str, args: &InstallArgs) {
    let split = gh.split_once('/').unwrap();
    let owner = split.0;
    let mut repo = split.1;
    let tag = if let Some((repo_, tag)) = repo.split_once('@') {
        repo = repo_;
        tag
    } else {
        todo!("Missing tag not yet supported")
    };
    let (url, name) = get_gh_asset_info(args, owner, repo, tag);
    install_core(&url, args, &name, repo);
}

fn install_url(url: &str, args: &InstallArgs) {
    let name = url.split('/').next_back().unwrap();
    let output_name = crate::guess::guess_binary_filename_from_url(url);
    install_core(url, args, name, &output_name);
}

/// Install a binary.
pub(crate) fn run(args: &InstallArgs) {
    if let Some(gh) = &args.gh {
        install_gh(gh, args);
    } else if let Some(url) = &args.url {
        install_url(url, args);
    } else {
        abort("Expected either `--gh` or `--url` to be specified");
    }
}
