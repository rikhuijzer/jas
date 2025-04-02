mod install;
mod sha;

use clap::Parser;
use sha::Sha256Hash;
use std::path::PathBuf;
use tracing::subscriber::SetGlobalDefaultError;
use tracing::Level;

#[derive(Clone, Debug, Parser)]
pub(crate) struct ShaArgs {
    #[arg(short, long)]
    gh: Option<String>,
    #[arg(short, long)]
    path: Option<String>,
}

#[derive(Clone, Debug, Parser)]
pub(crate) struct InstallArgs {
    #[arg(short, long)]
    gh: Option<String>,
    #[arg(short, long)]
    sha: Option<String>,
    #[arg(short, long, default_value = "~/.jas/bin")]
    dir: String,
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
    #[command(subcommand)]
    task: Task,

    #[arg(short, long)]
    verbose: Option<bool>,
}

/// Initialize logging with the given level.
pub fn init_subscriber(level: Level) -> Result<(), SetGlobalDefaultError> {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(level)
        .with_test_writer()
        .without_time()
        .with_target(false)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
}

#[tokio::main]
async fn main() {
    let args = Arguments::parse();
    let level = if let Some(verbose) = args.verbose {
        if verbose {
            Level::DEBUG
        } else {
            Level::INFO
        }
    } else {
        Level::INFO
    };
    init_subscriber(level).unwrap();

    match args.task {
        Task::Sha(args) => {
            if let Some(path) = &args.path {
                let path = PathBuf::from(path);
                if !path.exists() {
                    panic!("Path does not exist: {}", path.display());
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
