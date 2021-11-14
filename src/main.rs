use anyhow::Result;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug)]
struct CustomError(String);

/// Search for a pattern in a file and display the lines that contain it.
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
        #[structopt(short, long, default_value = ".")]
        target: String,
    },

    /// Apply changes
    Apply {
        /// The path to the file to read
        #[structopt(short, long, parse(from_os_str))]
        file: PathBuf,

        /// Dry run
        #[structopt(short, long)]
        dryrun: bool,
    },
}

fn main() -> Result<()> {
    match Cli::from_args().cmd {
        Command::Gen { target } => {
            println!("Generated to {}", target);
        },

        Command::Apply { file, dryrun } => {
            println!("Applying from {:?}, dry-run = {}", file, dryrun);
        },

    }

    Ok(())
}
