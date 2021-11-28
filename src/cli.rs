use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug)]
pub struct CustomError(String);

/// Manage database roles and privileges in GitOps style
#[derive(Debug, StructOpt)]
pub struct Cli {
    #[structopt(subcommand)]
    pub cmd: Command,
}

#[derive(StructOpt, Debug)]
pub enum Command {
    /// Generate project
    Gen {
        /// The target folder
        #[structopt(short, long, default_value = ".", parse(from_os_str))]
        target: PathBuf,
    },

    /// Generate random password
    GenPass {
        /// The target folder
        #[structopt(short, long, default_value = "16")]
        length: u8,
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

// Parse the command line arguments
pub fn parse() -> Cli {
    Cli::from_args()
}
