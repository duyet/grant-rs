use log::info;
use std::fs;
use std::path::PathBuf;

pub fn gen(target: &PathBuf) {
    fs::create_dir_all(target).unwrap_or_else(|_| panic!("failed to generate {:?}", target));
    info!("Generated to {:?}", target);
}
