use anyhow::Result;
use env_logger::Env;
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

fn main() -> Result<()> {
    // Logger config, for debugger export RUST_LOG=debug
    let env = Env::new().default_filter_or("info");
    env_logger::init_from_env(env);

    match Cli::from_args().cmd {
        Command::Gen { target } => {
            grant::gen::gen(&target);
        }

        Command::GenPass { length } => {
            grant::gen::gen_password(length);
        }

        Command::Apply { file, dryrun, conn } => {
            grant::apply::apply(&file, dryrun, conn);
        }
    }

    Ok(())
}
