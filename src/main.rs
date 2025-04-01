mod sha;

use clap::Parser;
use sha::Sha256Hash;
use std::path::PathBuf;

#[derive(Clone, Debug, Parser)]
pub(crate) struct ShaArgs {
    #[arg(short, long)]
    github: Option<String>,
    #[arg(short, long)]
    path: Option<String>,
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
            if let Some(path) = &args.path {
                let path = PathBuf::from(path);
                if !path.exists() {
                    panic!("Path does not exist: {}", path.display());
                }
                let digest = Sha256Hash::from_path(&path);
                println!("{}", digest);
            } else if let Some(_github) = args.github {
                todo!()
            } else {
                todo!()
            }
        }
    }
}
