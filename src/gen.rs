use crate::config::Config;
use ansi_term::Colour::Green;
use log::info;
use md5::compute;
use rand::Rng;
use std::fs;
use std::path::PathBuf;

/// Generate project template to given target
pub fn gen(target: &PathBuf) {
    // Skip if target already exists
    if target.exists() {
        info!("target already exists");
        return;
    }

    fs::create_dir_all(target).unwrap_or_else(|_| panic!("failed to generate {:?}", target));
    info!("creating path: {:?}", target);

    let config = Config::default();
    let config_str = serde_yaml::to_string(&config).unwrap();

    // Write config_str to target/config.yml
    let config_path = target.join("config.yml");
    fs::write(config_path.clone(), config_str)
        .unwrap_or_else(|_| panic!("failed to write {:?}", config_path));
    info!("Generated: {:?}", config_path);
}

/// Generating password with given length
pub fn gen_password(
    length: u8,
    no_special: bool,
    username: Option<String>,
    password: Option<String>,
) {
    // If not password is given, generate random password
    let password = match password {
        Some(p) => p,
        None => {
            let chars: &[u8] = if no_special {
                b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                  abcdefghijklmnopqrstuvwxyz\
                  0123456789"
            } else {
                b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                  abcdefghijklmnopqrstuvwxyz\
                  0123456789)(*&^%#!~"
            };
            let mut rng = rand::thread_rng();
            let password: String = (0..length)
                .map(|_| {
                    let idx = rng.gen_range(0..chars.len());
                    chars[idx] as char
                })
                .collect();

            password
        }
    };

    println!("Generated password: {}", Green.paint(password.clone()));

    if let Some(username) = username {
        let password_hash = gen_md5_password(&password, &username);
        println!(
            "Generated MD5 (user: {}): {}",
            username,
            Green.paint(password_hash)
        );
        println!("\nHint: https://docs.aws.amazon.com/redshift/latest/dg/r_CREATE_USER.html");
    } else {
        println!("\nHint: Please provide --username to generate MD5");
    }
}

/// Generate md5 password hash from username and password
/// 1. Concatenate the password and username
/// 2. Hash the concatenated string
/// 3. Concatenate 'md5' in front of the MD5 hash string
/// https://docs.aws.amazon.com/redshift/latest/dg/r_CREATE_USER.html
fn gen_md5_password(password: &str, username: &str) -> String {
    format!(
        "md5{:x}",
        compute(format!("{}{}", password, username).as_bytes())
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test gen_password
    #[test]
    fn test_gen_password() {
        gen_password(10, true, None, None);
        gen_password(10, true, Some("test".to_string()), None);
        gen_password(10, true, Some("test".to_string()), Some("test".to_string()));
        gen_password(10, false, None, None);
        gen_password(10, false, Some("test".to_string()), None);
        gen_password(
            10,
            false,
            Some("test".to_string()),
            Some("test".to_string()),
        );
    }

    // Test gen_md5_password
    #[test]
    fn test_gen_md5_password() {
        assert_eq!(
            gen_md5_password("test", "test"),
            "md505a671c66aefea124cc08b76ea6d30bb"
        );
    }
}
