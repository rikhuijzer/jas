mod sha;

use clap::Parser;

#[derive(Clone, Debug, Parser)]
pub(crate) struct ShaArgs {
    #[arg(short, long)]
    github: Option<String>,
    #[arg(short, long)]
    sha: Option<String>,
}

#[derive(Clone, Debug, clap::Subcommand)]
pub(crate) enum Task {
    Sha(ShaArgs),
}

#[derive(Clone, Debug, Parser)]
#[command(author, version, about)]
pub(crate) struct Arguments {
    #[command(subcommand)]
    task: Task,
}

fn main() {
    let args = Arguments::parse();

    match args.task {
        Task::Sha(args) => {
            println!("SHA: {}", args.sha.unwrap_or_default());
        }
    }
}
