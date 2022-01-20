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
    /// Generate sample configuration file
    Gen {
        /// The target folder
        #[structopt(short, long, default_value = ".", parse(from_os_str))]
        target: PathBuf,
    },

    /// Generate random password
    GenPass {
        /// The target folder
        #[structopt(short, long, default_value = "32")]
        length: u8,
        /// No special characters
        #[structopt(short, long)]
        no_special: bool,
        /// The username, using to create md5 hash
        #[structopt(short, long)]
        username: Option<String>,
        /// The password, using to create md5 hash
        #[structopt(short, long)]
        password: Option<String>,
    },

    /// Apply a configuration to a redshift by file name.
    /// Yaml format are accepted.
    Apply {
        /// The path to the file to read, directory is not supported yet.
        #[structopt(short, long, parse(from_os_str))]
        file: PathBuf,

        /// Dry run mode, only print what would be apply
        #[structopt(short, long)]
        dryrun: bool,

        /// Apply all files in the current folder or target folder (if --file is a folder)
        #[structopt(short, long)]
        all: bool,
    },

    /// Validate a configuration file or
    /// a target directory that contains configuration files
    Validate {
        /// The path to the file or directory
        /// If the target is not available, the current
        /// directory will be used.
        #[structopt(short, long, parse(from_os_str))]
        file: Option<PathBuf>,
    },

    /// Inspect current database cluster
    /// with connection info from configuration file
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
