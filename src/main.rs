mod apply;
mod cli;
mod config;
mod connection;
mod gen;
mod inspect;
mod validate;

use crate::config::Config;
use anyhow::Result;
use cli::Command;
use env_logger::Env;
use std::path::PathBuf;

fn main() -> Result<()> {
    // Logger config, for debugger export RUST_LOG=debug
    let env = Env::new().default_filter_or("info");
    env_logger::init_from_env(env);

    match cli::parse().cmd {
        Command::Gen { target } => {
            gen::gen(&target);
        }

        Command::GenPass {
            length,
            username,
            password,
        } => {
            gen::gen_password(length, username, password);
        }

        Command::Validate { file } => {
            let target = if let Some(file) = file {
                file
            } else {
                PathBuf::from(std::env::current_dir()?)
            };

            validate::validate_target(&target)?;
        }

        Command::Inspect { file } => {
            let value = Config::new(&file)?;
            inspect::inspect(&value)?;
        }

        Command::Apply { file, dryrun, .. } => {
            let value = Config::new(&file)?;
            apply::apply(&value, dryrun)?;
        }
    }

    Ok(())
}
