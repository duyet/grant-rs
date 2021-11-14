extern crate yaml_rust;

use anyhow::{Context, Result};
use indoc::indoc;
use std::fs;
use std::path::PathBuf;
use yaml_rust::{Yaml, YamlLoader};
use yaml_validator::{Context as C, Validate};

pub mod gen;

/// Get file content
pub fn get_content(path: &PathBuf) -> Result<String> {
    let content =
        fs::read_to_string(path).with_context(|| format!("could not read file `{:?}`", path))?;

    Ok(content)
}

/// Read yaml config file and assert schema
pub fn read_config(path: &PathBuf) -> Result<Yaml> {
    let content = get_content(path)?;
    let config = YamlLoader::load_from_str(&content)
        .expect("could not parse YAML")
        .remove(0);

    validate_schema(&config);

    Ok(config)
}

/// Validate YAML with schema
pub fn validate_schema(yaml: &Yaml) {
    let schema_yaml = YamlLoader::load_from_str(indoc! { r#"
        ---
        uri: root
        schema:
          type: object
          items:
            roles:
              type: array
              # items:
              #   anyOf:
              #     $ref: role-database
              #     $ref: role-schema
              #     $ref: role-table
              #     $ref: role-func-or-procn
              #     $ref: role-language
            users:
              type: array
              items:
                $ref: user

        ---
        uri: role-database
        schema:
          type: object
          items:
            name:
              type: string
            level:
              type: string
            grants:
              $ref: type-array-string
            databases:
              $ref: type-array-string
        ---
        uri: role-schema
        schema:
          type: object
          items:
            name:
              type: string
            level:
              type: string
            grants:
              $ref: type-array-string
            databases:
              $ref: type-array-string
            schemas:
              $ref: type-array-string
        ---
        uri: role-table
        schema:
          type: object
          items:
            name:
              type: string
            level:
              type: string
            grants:
              $ref: type-array-string
            databases:
              $ref: type-array-string
            schemas:
              $ref: type-array-string
            tables:
              $ref: type-array-string
        ---
        uri: role-func-or-proc
        schema:
          type: object
          items:
            name:
              type: string
            level:
              type: string
            grants:
              $ref: type-array-string
            databases:
              $ref: type-array-string
            schemas:
              $ref: type-array-string
            functions:
              $ref: type-array-string
        ---
        uri: role-language
        schema:
          type: object
          items:
            name:
              type: string
            level:
              type: string
            grants:
              $ref: type-array-string
            languages:
              $ref: type-array-string
        ---
        uri: type-array-string
        schema:
          type: array
          items:
            type: string
        ---
        uri: user
        schema:
          type: object
          items:
            name:
              type: string
            roles:
              type: array
              items:
                type: object
                items:
                  name:
                    type: string
        "#})
    .unwrap();

    // println!("{:#?}", schema_yaml);

    let context = C::try_from(&schema_yaml).unwrap();
    let schema = context.get_schema("root").unwrap();
    schema.validate(&context, &yaml).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
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

    // Test config with valid YAML
    #[test]
    fn test_read_config_basic_config() {
        let _text = indoc! {"
            roles: []
            users: []
        "};

        let mut file = NamedTempFile::new().expect("failed to create temp file");
        file.write(_text.as_bytes())
            .expect("failed to write to temp file");
        let path = PathBuf::from(file.path().to_str().unwrap());

        let config = read_config(&path).unwrap();

        assert!(config["roles"].as_vec().unwrap().len() == 0);
        assert!(config["users"].as_vec().unwrap().len() == 0);
    }

    #[test]
    fn test_read_config_full_config() {
        let _text = indoc! {"
            roles:
              - name: role_database_level
                level: DATABASE
                grants:
                  - CREATE
                  - TEMP
                databases:
                  - db1
                  - db2
              - name: role_schema_level
                level: SCHEMA
                grants:
                  - CREATE
                  - USAGE
                databases:
                  - db1
                  - db2
                schemas:
                  - common
                  - dwh1
                  - dwh2
              - name: role_all_schema
                level: SCHEMA
                grants:
                  - CREATE
                  - USAGE
                databases:
                  - db1
                  - db2

            users:
              - name: duyet
                roles:
                  - name: role_database_level
                  - name: role_all_schema
                  - name: role_schema_level
        "};

        let mut file = NamedTempFile::new().expect("failed to create temp file");
        file.write(_text.as_bytes())
            .expect("failed to write to temp file");
        let path = PathBuf::from(file.path().to_str().unwrap());

        let config = read_config(&path).unwrap();

        assert!(config["roles"].as_vec().unwrap().len() == 3);
        assert!(config["users"].as_vec().unwrap().len() == 1);
    }

    // Test read_config with invalid YAML
    #[test]
    #[should_panic]
    fn test_read_config_invalid_config() {
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

        // should panic
        read_config(&path).unwrap();
    }
}
