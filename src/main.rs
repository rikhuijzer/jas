mod guess;
mod install;
mod sha;

use clap::Parser;
use sha::Sha256Hash;
use std::path::PathBuf;
use tracing::subscriber::SetGlobalDefaultError;
use tracing::Level;

#[derive(Clone, Debug, Parser)]
pub(crate) struct ShaArgs {
    /// The GitHub repository to compute the SHA-256 hash of
    #[arg(short, long)]
    gh: Option<String>,
    /// The file path to compute the SHA-256 hash of
    #[arg(short, long)]
    path: Option<String>,
}

fn release_mode() -> bool {
    cfg!(not(debug_assertions))
}

pub(crate) fn abort(message: &str) -> ! {
    if release_mode() {
        tracing::error!("{message}");
        std::process::exit(1);
    } else {
        panic!("{message}");
    }
}

#[derive(Clone, Debug, Parser)]
pub(crate) struct InstallArgs {
    /// The GitHub repository to install from
    #[arg(long)]
    gh: Option<String>,
    /// The URL to install from
    #[arg(long)]
    url: Option<String>,
    /// The SHA-256 hash of the binary to install [default: no verification]
    #[arg(long)]
    sha: Option<String>,
    /// The directory to install the binary to
    #[arg(long, default_value = "~/.jas/bin")]
    dir: String,
    /// The name of the GitHub release asset to install
    #[arg(long)]
    asset_name: Option<String>,
    /// The name of the binary after installation [default: the repo name or guessed from the url]
    #[arg(long)]
    binary_filename: Option<String>,
    /// The name of the binary in the archive [default: use simple heuristic to guess]
    #[arg(long)]
    archive_filename: Option<String>,
}

#[derive(Clone, Debug, clap::Subcommand)]
pub(crate) enum Task {
    /// Compute the SHA-256 hash of a file or a GitHub repository.
    Sha(ShaArgs),
    /// Install a binary from a GitHub repository.
    Install(InstallArgs),
}

#[derive(Clone, Debug, Parser)]
#[command(author, version, about)]
pub(crate) struct Arguments {
    /// Whether to print verbose output
    #[arg(long)]
    verbose: bool,

    /// Whether to use ANSI escape codes
    #[arg(long, default_value = "true")]
    ansi: Option<bool>,

    #[command(subcommand)]
    task: Task,
}

/// Initialize logging with the given level.
pub fn init_subscriber(level: Level, ansi: bool) -> Result<(), SetGlobalDefaultError> {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(level)
        .with_test_writer()
        .without_time()
        .with_target(false)
        .with_ansi(ansi)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
}

#[tokio::main]
async fn main() {
    let args = Arguments::parse();
    let level = if args.verbose {
        Level::DEBUG
    } else {
        Level::INFO
    };
    init_subscriber(level, args.ansi.unwrap_or(true)).unwrap();

    match args.task {
        Task::Sha(args) => {
            if let Some(path) = &args.path {
                let path = PathBuf::from(path);
                if !path.exists() {
                    abort(&format!("Path does not exist: {}", path.display()));
                }
                let digest = Sha256Hash::from_path(&path);
                println!("{}", digest);
            } else if let Some(_github) = args.gh {
                todo!()
            } else {
                todo!()
            }
        }
        Task::Install(args) => {
            install::install(&args).await;
        }
    }
}
