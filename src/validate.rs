use crate::config::Config;
use ansi_term::Colour::{Green, Red};
use anyhow::{anyhow, Result};
use std::path::PathBuf;
use walkdir::WalkDir;

/// Validate the target PathBuf
pub fn validate_target(target: &PathBuf) -> Result<()> {
    if !target.exists() {
        return Err(anyhow!(
            "{:?} ... {} - file/directory does not exist",
            target,
            Red.paint("Failed")
        ));
    }

    // Scan all files recursive from current directory
    // that match *.yaml or *.yml and validate them
    if target.is_dir() {
        let mut files = vec![];
        for entry in WalkDir::new(target) {
            let entry = entry?;
            if entry.path().is_file() {
                let file_name = entry.path().file_name().unwrap();
                if file_name.to_str().unwrap().ends_with(".yaml")
                    || file_name.to_str().unwrap().ends_with(".yml")
                {
                    let path = entry.path().to_path_buf();
                    files.push(path);
                }
            }
        }

        for file in files {
            // Validate but not panic
            validate_file(&file).unwrap_or_else(|e| {
                println!("{}", e);
            });
        }

        return Ok(());
    }

    // Validate single file
    Ok(validate_file(target)?)
}

/// Validate target yaml file
pub fn validate_file(file: &PathBuf) -> Result<()> {
    let value = Config::new(file)
        .map_err(|e| anyhow!("{:?} ... {} - {}", file, Red.paint("invalid"), e))?;

    value
        .validate()
        .map_err(|e| anyhow!("{:?} ... {} - {}", file, Red.paint("invalid"), e))?;

    // "OK" in green color
    println!("{:?} ... {}", file, Green.paint("ok"));

    Ok(())
}
