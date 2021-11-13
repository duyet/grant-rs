use std::fs;
use std::path::PathBuf;

pub fn gen<E: std::convert::From<std::io::Error>>(target: &PathBuf) -> Result<(), E> {
    fs::create_dir_all(target)?;

    Ok(())
}
