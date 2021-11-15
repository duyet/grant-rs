use log::info;
use rand::Rng;
use std::fs;
use std::path::PathBuf;

/// Generate project template to given target
pub fn gen(target: &PathBuf) {
    fs::create_dir_all(target).unwrap_or_else(|_| panic!("failed to generate {:?}", target));
    info!("Generated to {:?}", target);
}

/// Generated password with given length
pub fn gen_password(length: u8) {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789)(*&^%$#@!~";

    let mut rng = rand::thread_rng();

    let password: String = (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();

    println!("Generated password: {}", password);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    // Test gen_password
    fn test_gen_password() {
        gen_password(10);
    }
}
