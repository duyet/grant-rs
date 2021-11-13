extern crate yaml_rust;

use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;
use yaml_rust::{Yaml, YamlLoader};

pub mod gen;

pub fn get_content(path: &PathBuf) -> Result<String> {
    let content =
        fs::read_to_string(path).with_context(|| format!("could not read file `{:?}`", path))?;

    Ok(content)
}

pub fn read_config(path: &PathBuf) -> Result<Vec<Yaml>> {
    let content = get_content(path)?;
    let config = YamlLoader::load_from_str(&content).expect("could not parse YAML");

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_get_content() {
        let _text = "content";
        let mut file = NamedTempFile::new().expect("failed to create temp file");
        file.write(_text.as_bytes())
            .expect("failed to write to temp file");
        let path = PathBuf::from(file.path().to_str().unwrap());

        let content = get_content(&path).expect("failed to get content");
        assert_eq!(_text, content);
    }

    // Test read_config with random YAML content
    #[test]
    fn test_read_config() {
        let _text = indoc! {"
            a: 1
            b: str
            c:
            d:
              - x1
              - x2
            e:
              x: 1
              y: 2
            f:
              x: 1
              y: 2"};

        let mut file = NamedTempFile::new().expect("failed to create temp file");
        file.write(_text.as_bytes())
            .expect("failed to write to temp file");
        let path = PathBuf::from(file.path().to_str().unwrap());

        let config = read_config(&path).unwrap();
        println!("{:?}", config);

        assert!(config[0]["a"].as_i64().unwrap() == 1);
        assert!(config[0]["b"].as_str().unwrap() == "str");
        assert!(config[0]["c"] == Yaml::Null);
        assert!(
            config[0]["d"]
                == Yaml::Array(vec![
                    Yaml::String("x1".to_string()),
                    Yaml::String("x2".to_string())
                ])
        );
        assert!(config[0]["e"]["x"] == Yaml::Integer(1));
        assert!(config[0]["e"]["y"] == Yaml::Integer(2));
    }
}
