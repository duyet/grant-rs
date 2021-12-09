use assert_cmd::prelude::*; // Add methods on commands
use indoc::indoc;
use predicates::prelude::*; // Used for writing assertions
use std::io::Write; // Write to files
use std::path::PathBuf;
use std::process::Command; // Run programs
use tempfile::NamedTempFile; // Create temporary files

#[test]
fn missing_arguments() {
    let mut cmd = Command::cargo_bin("grant").unwrap();
    cmd.assert().failure();
}

#[test]
/// `grant apply` must have --file or -f args
fn apply_missing_arguments() {
    let mut cmd = Command::cargo_bin("grant").unwrap();
    cmd.arg("apply")
        .assert()
        .failure()
        .stderr(predicate::str::contains("--file"));
}

/// `grant apply` with a config file
#[test]
fn apply_with_config_file() {
    // create a config file
    let _text = indoc! {r#"
        connection:
          type: "postgres"
          url: "postgres://postgres:postgres@localhost:5432/postgres"

        roles:
          - name: role_database_level
            type: database
            grants:
              - CREATE
              - TEMP
            databases:
              - postgres

          - name: role_schema_level
            type: schema
            grants:
              - CREATE
            databases:
              - postgres
            schemas:
              - public
          - name: role_all_schema
            type: table
            grants:
              - SELECT
              - INSERT
              - UPDATE
            databases:
              - postgres
            schemas:
              - public
            tables:
              - ALL

        users:
          - name: duyet
            password: 1234567890
            roles:
              - role_database_level
              - role_all_schema
              - role_schema_level
          - name: duyet2
            password: 1234567890
            roles:
              - role_database_level
              - role_all_schema
              - role_schema_level
    "#};

    let mut file = NamedTempFile::new().expect("failed to create temp file");
    file.write(_text.as_bytes())
        .expect("failed to write to temp file");
    let path = PathBuf::from(file.path().to_str().unwrap());

    let mut cmd = Command::cargo_bin("grant").unwrap();
    cmd.arg("apply")
        .arg("--file")
        .arg(path)
        .assert()
        .success()
        .stderr(predicate::str::contains(
            "Connected to database: postgres://postgres:postgres@localhost:5432/postgres",
        ))
        .stderr(predicate::str::contains("duyet"))
        .stderr(predicate::str::contains("duyet2"))
        // Look like this:
        // ┌────────┬───────────────────────────────────────────────────────────┬─────────┐
        // │ User   │ Database Privilege                                        │ Action  │
        // │ ---    │ ---                                                       │ ---     │
        // │ duyet  │ `role_database_level` for database: ["postgre+ │ updated │
        // │ duyet2 │ `role_database_level` for database: ["postgre+ │ updated │
        // └────────┴───────────────────────────────────────────────────────────┴─────────┘
        .stderr(predicate::str::contains(
            "│ duyet  │ `role_database_level` for database",
        ))
        .stderr(predicate::str::contains(
            "│ duyet2 │ `role_database_level` for database",
        ))
        // Look like this:
        // ┌────────┬───────────────────────────────────────────────────────┬─────────┐
        // │ User   │ Schema Privileges                                     │ Action  │
        // │ ---    │ ---                                                   │ ---     │
        // │ duyet  │ `role_schema_level` for schema: ["public"] │ updated │
        // │ duyet2 │ `role_schema_level` for schema: ["public"] │ updated │
        // └────────┴───────────────────────────────────────────────────────┴─────────┘
        .stderr(predicate::str::contains(
            "│ duyet  │ `role_schema_level` for schema",
        ))
        .stderr(predicate::str::contains(
            "│ duyet2 │ `role_schema_level` for schema",
        ))
        // Look like this:
        // ┌────────┬─────────────────────────────────────────────────┬─────────┐
        // │ User   │ Table Privileges                                │ Action  │
        // │ ---    │ ---                                             │ ---     │
        // │ duyet  │ `role_all_schema` for table: ["ALL"] │ updated │
        // │ duyet2 │ `role_all_schema` for table: ["ALL"] │ updated │
        // └────────┴─────────────────────────────────────────────────┴─────────┘
        .stderr(predicate::str::contains(
            "│ duyet  │ `role_all_schema` for table",
        ))
        .stderr(predicate::str::contains(
            "│ duyet2 │ `role_all_schema` for table",
        ))
        .stderr(predicate::str::contains("Summary"));
}
