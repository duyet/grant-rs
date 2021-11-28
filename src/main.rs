mod apply;
mod cli;
mod config;
mod gen;

use crate::config::Config;
use anyhow::Result;
use cli::Command;
use env_logger::Env;

fn main() -> Result<()> {
    // Logger config, for debugger export RUST_LOG=debug
    let env = Env::new().default_filter_or("info");
    env_logger::init_from_env(env);

    match cli::parse().cmd {
        Command::Gen { target } => {
            gen::gen(&target);
        }

        Command::GenPass { length } => {
            gen::gen_password(length);
        }

        Command::Apply { file, dryrun, .. } => {
            let value = Config::new(&file)?;
            apply::apply(&value, dryrun)?;
        }
    }

    Ok(())
}
