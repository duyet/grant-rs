use anyhow::Result;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug)]
struct CustomError(String);

/// Manage database roles and privileges in GitOps style
#[derive(Debug, StructOpt)]
struct Cli {
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(StructOpt, Debug)]
enum Command {
    /// Generate project
    Gen {
        /// The target folder
        #[structopt(short, long, default_value = ".", parse(from_os_str))]
        target: PathBuf,
    },

    /// Apply changes
    Apply {
        /// The path to the file to read
        #[structopt(short, long, parse(from_os_str))]
        file: PathBuf,

        /// Dry run
        #[structopt(short, long)]
        dryrun: bool,

        /// Connection string
        #[structopt(short, long)]
        conn: Option<String>,
    },
}

fn main() -> Result<()> {
    match Cli::from_args().cmd {
        Command::Gen { target } => {
            grant::gen::gen(&target);
        }

        Command::Apply { file, dryrun, conn } => {
            println!(
                "Applying from {:?}, dry-run = {}, conn = {:?}",
                file, dryrun, conn
            );
        }
    }

    Ok(())
}
