mod guess;
mod install;
mod sha;

use clap::Parser;
use tracing::subscriber::SetGlobalDefaultError;
use tracing::Level;

#[derive(Clone, Debug, Parser)]
pub(crate) struct ShaArgs {
    /// The file path to compute the SHA-256 hash of
    #[arg(short, long)]
    path: Option<String>,
    /// The URL to compute the SHA-256 hash of
    #[arg(short, long)]
    url: Option<String>,
}

fn release_mode() -> bool {
    cfg!(not(debug_assertions))
}

fn is_ci() -> bool {
    std::env::var("CI").unwrap_or("false".to_string()) == "true"
}

pub(crate) fn abort(message: &str) -> ! {
    if release_mode() || is_ci() {
        tracing::error!("{message}");
        std::process::exit(1);
    } else {
        panic!("{message}");
    }
}

static DEFAULT_INSTALL_DIR: &str = "~/.jas/bin";

#[derive(Clone, Debug, Parser)]
pub(crate) struct InstallArgs {
    /// The GitHub repository to install from
    ///
    /// For example, `crate-ci/typos@v1.31.1`.
    #[arg(long)]
    gh: Option<String>,
    /// The GitHub token to use
    ///
    /// This is usually desired inside GitHub Actions because otherwise this
    /// tool might be rate limited when determining which assets are available
    /// in the release. The limit is 60 requests per hour per IP address. For
    /// home-use this should normally not be an issue, but if you are using a
    /// shared system such as GitHub Runners it will. If you don't want to use a
    /// GITHUB_TOKEN it is also possible to avoid this issue by manually
    /// specifying the `--url` instead of `--gh`.
    ///
    /// As a side-note, GitHub Actions normally avoid this problem due to them
    /// receiving the token by default according to the GitHub Docs:
    ///
    /// "An action can access the GITHUB_TOKEN through the github.token context
    /// even if the workflow does not explicitly pass the GITHUB_TOKEN to the
    /// action. As a good security practice, you should always make sure that
    /// actions only have the minimum access they require by limiting the
    /// permissions granted to the GITHUB_TOKEN. [...]"
    ///
    /// So this means that if you for example write,
    ///
    /// ```yaml
    /// - uses: tj-actions/changed-files@v40
    /// ```
    ///
    /// then this Action will have access to the GITHUB_TOKEN via the `github.token` context.
    #[arg(long, env = "GITHUB_TOKEN", verbatim_doc_comment)]
    gh_token: Option<String>,
    /// The URL to install from
    ///
    /// For example, "github.com/crate-ci/typos/releases/download/v1.31.1/typos-v1.31.1-x86_64-unknown-linux-musl.tar.gz".
    #[arg(long)]
    url: Option<String>,
    /// The SHA-256 hash of the binary to install
    ///
    /// [default: no verification if no hash is provided]
    #[arg(long)]
    sha: Option<String>,
    /// The directory to install the binary to
    #[arg(long, default_value = DEFAULT_INSTALL_DIR)]
    dir: String,
    /// The name of the GitHub release asset to install
    #[arg(long)]
    asset_name: Option<String>,
    /// The name of the binary/binaries in the archive
    ///
    /// [default: use simple heuristic to guess]
    #[arg(long)]
    archive_filename: Option<Vec<String>>,
    /// The name of the executable/executables after installation
    ///
    /// [default: the repo name or guessed from the url]
    #[arg(long)]
    executable_filename: Option<Vec<String>>,
}

#[derive(Clone, Debug, Parser)]
pub(crate) struct ShowArgs {
    /// Print the default install directory.
    #[arg(long)]
    install_dir: bool,
}

#[derive(Clone, Debug, clap::Subcommand)]
pub(crate) enum Task {
    /// Compute the SHA-256 hash of a file or a GitHub repository.
    Sha(ShaArgs),
    /// Install a binary from a GitHub repository.
    Install(InstallArgs),
    /// Show information.
    Show(ShowArgs),
    /// Print the project's license.
    License,
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
        // Write logs to stderr to allow writing sha output to stdout.
        .with_writer(std::io::stderr)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
}

fn main() {
    let args = Arguments::parse();
    let level = if args.verbose {
        Level::DEBUG
    } else {
        Level::INFO
    };
    init_subscriber(level, args.ansi.unwrap_or(true)).unwrap();

    match args.task {
        Task::Sha(args) => {
            sha::run(&args);
        }
        Task::Install(args) => {
            install::run(&args);
        }
        Task::Show(args) => {
            if args.install_dir {
                let path = install::interpret_path(DEFAULT_INSTALL_DIR);
                println!("{}", path.display());
            }
        }
        Task::License => {
            println!("{}", include_str!("../LICENSE"));
        }
    }
}
