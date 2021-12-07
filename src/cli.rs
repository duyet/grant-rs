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
        /// The username, using to create md5 hash
        #[structopt(short, long)]
        username: Option<String>,
        /// The password, using to create md5 hash
        #[structopt(short, long)]
        password: Option<String>,
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

    /// Validate target file
    Validate {
        /// The path to the file to read (optional)
        #[structopt(short, long, parse(from_os_str))]
        file: Option<PathBuf>,
    },

    /// Inspect current database cluster by config file
    Inspect {
        /// The path to the file to read
        #[structopt(short, long, parse(from_os_str))]
        file: PathBuf,
    },
}

// Parse the command line arguments
pub fn parse() -> Cli {
    Cli::from_args()
}
