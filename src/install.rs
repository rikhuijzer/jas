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
    if let Some(token) = &args.gh_token {
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

pub(crate) fn interpret_path(path: &str) -> PathBuf {
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

fn is_tar_gz(name: &str) -> bool {
    name.ends_with(".tar.gz") || name.ends_with(".tgz")
}

/// Unpack a gzipped archive into a directory.
fn unpack_archive(body: &[u8], dir: &Path, name: &str) -> Option<PathBuf> {
    let stem = Path::new(name).file_stem();
    let archive_dir = dir.join(stem.as_ref().unwrap());
    if is_tar_gz(name) || name.ends_with(".tar.xz") || name.ends_with(".zip") {
        if archive_dir.exists() {
            if archive_dir.is_dir() {
                std::fs::remove_dir_all(&archive_dir).unwrap();
            } else {
                std::fs::remove_file(&archive_dir).unwrap();
            }
        }
        std::fs::create_dir_all(&archive_dir).unwrap();
    }
    if is_tar_gz(name) {
        std::fs::create_dir_all(&archive_dir).unwrap();
        let decompressed = GzDecoder::new(body);
        let mut archive = Archive::new(decompressed);
        archive.unpack(&archive_dir).unwrap();
        tracing::debug!("Unpacked archive into {}", archive_dir.display());
        Some(archive_dir)
    } else if name.ends_with(".tar.xz") {
        let decompressed = xz2::read::XzDecoder::new(body);
        let mut archive = Archive::new(decompressed);
        archive.unpack(&archive_dir).unwrap();
        tracing::debug!("Unpacked archive into {}", archive_dir.display());
        Some(archive_dir)
    } else if name.ends_with(".zip") {
        use zip::unstable::stream::ZipStreamReader;
        let archive_dir = dir.join(name.strip_suffix(".zip").unwrap());
        let zip = ZipStreamReader::new(body);
        zip.extract(&archive_dir).unwrap();
        tracing::debug!("Unpacked archive into {}", archive_dir.display());
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

fn add_exe_if_needed(path: &Path) -> PathBuf {
    if cfg!(target_os = "windows") {
        // File could be a .py script, so don't add .exe.
        if path.extension().is_some() {
            path.to_path_buf()
        } else {
            path.with_extension("exe")
        }
    } else {
        path.to_path_buf()
    }
}

/// If the archive contains a bin directory, ignore all other files.
fn filter_if_bin(files: &mut Vec<PathBuf>) {
    let has_bin = files
        .iter()
        .position(|f| f.file_name().is_some_and(|s| s.to_str() == Some("bin")));
    if let Some(has_bin) = has_bin {
        tracing::debug!("has_bin: {has_bin:?}");
        tracing::debug!("files: {}", files[has_bin].display());
        *files = vec![files[has_bin].clone()];
    }
}

/// Return the files in an archive.
///
/// Also handles archives with nested directories.
fn files_in_archive(archive_dir: &Path) -> Vec<PathBuf> {
    let files = std::fs::read_dir(archive_dir)
        .unwrap_or_else(|_| abort(&format!("Failed to read directory at {archive_dir:?}")));
    let mut files = files.map(|file| file.unwrap().path()).collect::<Vec<_>>();
    filter_if_bin(&mut files);
    // If the archive contains a single dir, read the files in that dir.
    if files.len() == 1 {
        let path = &files[0];
        if path.is_dir() {
            let path = archive_dir.join(path.file_name().unwrap());
            files_in_archive(&path)
        } else {
            files
        }
    } else {
        files
    }
}

fn verify_filenames_match(filenames: &[String], executable_filenames: &[String]) {
    if filenames.len() != executable_filenames.len() {
        abort(&format!(
            "Expected {} executable filenames, got {}",
            executable_filenames.len(),
            filenames.len()
        ));
    }
}

/// Return (src, dst) pairs for each `filename` in `archive_filename`.
fn handle_filenames(
    dir: &Path,
    archive_dir: &Path,
    args: &InstallArgs,
    filenames: &[String],
) -> Vec<(PathBuf, PathBuf)> {
    let executable_filename = if let Some(executable_filenames) = &args.executable_filename {
        verify_filenames_match(filenames, executable_filenames);
        Some(executable_filenames[0].clone())
    } else {
        None
    };
    filenames
        .iter()
        .map(|filename| {
            let filename = add_exe_if_needed(Path::new(filename));
            let files = files_in_archive(archive_dir);
            let executable = files
                .iter()
                .find(|file| file.file_name() == filename.file_name());
            if let Some(executable) = executable {
                let src = executable.to_path_buf();
                let dst = if let Some(executable_filename) = &executable_filename {
                    let dst = add_exe_if_needed(Path::new(executable_filename));
                    dir.join(dst)
                } else {
                    let dst = add_exe_if_needed(Path::new(&filename));
                    dir.join(dst)
                };
                (src, dst)
            } else {
                abort(&format!(
                    "Could not find executable in archive; file {} not in\n{}",
                    filename.display(),
                    files
                        .iter()
                        .map(|f| f.display().to_string())
                        .collect::<Vec<_>>()
                        .join("\n")
                ));
            }
        })
        .collect::<Vec<_>>()
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

/// Copy the binary from the archive to the target directory.
fn copy_from_archive(dir: &Path, archive_dir: &Path, args: &InstallArgs, name: &str) {
    let src_dst = if let Some(filenames) = &args.archive_filename {
        handle_filenames(dir, archive_dir, args, filenames)
    } else {
        let files = files_in_archive(archive_dir);
        let src = crate::guess::guess_executable_in_archive(&files, name);
        let dst = if let Some(executable_filenames) = &args.executable_filename {
            if executable_filenames.len() != 1 {
                abort(
                    "Multiple `executable_filename`s can only be specified with multiple `archive_filename`s"
                );
            }
            dir.join(executable_filenames[0].clone())
        } else {
            dir.join(name)
        };
        let dst = add_exe_if_needed(&dst);
        vec![(src, dst)]
    };
    for (src, dst) in src_dst {
        let mut reader =
            File::open(&src).unwrap_or_else(|_| panic!("Failed to open binary at {src:?}"));
        let mut writer =
            File::create(&dst).unwrap_or_else(|_| panic!("Failed to create executable at {dst:?}"));
        std::io::copy(&mut reader, &mut writer).unwrap();
        tracing::info!("Placed binary at {}", dst.display());
        make_executable(&dst);
    }
}

fn download_file_core(url: &str) -> Result<Vec<u8>, String> {
    let mut response = match ureq::get(url).call() {
        Ok(response) => response,
        Err(e) => return Err(format!("Error downloading {url}: {e}")),
    };
    let limit_in_megabytes = 300;
    let limit = limit_in_megabytes * 1024 * 1024;
    match response.body_mut().with_config().limit(limit).read_to_vec() {
        Ok(body) => Ok(body),
        Err(e) => Err(format!("Error reading {url}: {e}")),
    }
}

pub(crate) fn download_file(url: &str) -> Vec<u8> {
    tracing::info!("Downloading {}", url);
    // Manual retry logic since ureq "3.x has no built-in retries".
    let retries = 3;
    for i in 0..retries {
        match download_file_core(url) {
            Ok(body) => return body,
            Err(e) => {
                if e.contains("timeout") && i < retries - 1 {
                    let wait = i * i + 1;
                    tracing::warn!("Timeout downloading {url}, retrying in {wait} seconds");
                    std::thread::sleep(std::time::Duration::from_secs(wait));
                } else {
                    abort(&format!("Error downloading {url}: {e}"));
                }
            }
        }
    }
    abort(&format!("Error downloading {url}: timeout"));
}

fn copy_file(body: &[u8], dir: &Path, output_name: &str) {
    let path = dir.join(output_name);
    let mut file = File::create(&path).unwrap();
    file.write_all(body).unwrap();
    make_executable(&path);
}

fn install_core(url: &str, args: &InstallArgs, name: &str, output_name: &str) {
    let body = download_file(url);
    verify_sha(&body, args);
    let dir = interpret_path(&args.dir);
    std::fs::create_dir_all(&dir).unwrap();
    let archive_dir = unpack_archive(&body, &dir, name);
    if let Some(archive_dir) = archive_dir {
        copy_from_archive(&dir, &archive_dir, args, output_name);
    } else {
        copy_file(&body, &dir, output_name);
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
    // Run the check here to error before download.
    if let Some(executable_filenames) = &args.executable_filename {
        if let Some(archive_filenames) = &args.archive_filename {
            verify_filenames_match(archive_filenames, executable_filenames);
        }
    }

    if let Some(gh) = &args.gh {
        install_gh(gh, args);
    } else if let Some(url) = &args.url {
        install_url(url, args);
    } else {
        abort("Expected either `--gh` or `--url` to be specified");
    }
}
