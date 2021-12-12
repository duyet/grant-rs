use assert_cmd::prelude::*; // Add methods on commands
use indoc::indoc;
use predicates::prelude::*; // Used for writing assertions
use std::io::Write;
use std::path::PathBuf;
use std::process::Command; // Run programs
use tempfile::NamedTempFile;

// Test the validate command
#[test]
/// `./grant validate` must have --file or -f args
fn validate_missing_arguments() {
    let mut cmd = Command::cargo_bin("grant").unwrap();
    cmd.arg("apply")
        .assert()
        .failure()
        .stderr(predicate::str::contains("--file"));
}

/// `./grant validate --file` must have a valid file
#[test]
fn validate_file_not_found() {
    let mut cmd = Command::cargo_bin("grant").unwrap();
    cmd.arg("apply")
        .arg("--file")
        .arg("/tmp/test-file-not-found")
        .assert()
        .failure();
}

/// Test the validate command with a valid file
#[test]
fn validate_file_valid() {
    let _text = indoc! {"
        connection:
          type: postgres
          url: postgres://postgres:postgres@localhost:5432/postgres

        roles:
          - name: role_database_level
            type: database
            grants:
              - CREATE
              - TEMP
            databases:
              - db1
              - db2

          - name: role_schema_level
            type: schema
            grants:
              - CREATE
            databases:
              - db1
              - db2
            schemas:
              - common
              - dwh1
              - dwh2

          - name: role_all_schema
            type: table
            grants:
              - SELECT
              - INSERT
              - UPDATE
            databases:
              - db1
            schemas:
              - common
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
             "};

    let mut file = NamedTempFile::new().expect("failed to create temp file");
    file.write(_text.as_bytes())
        .expect("failed to write to temp file");
    let path = PathBuf::from(file.path().to_str().unwrap());

    let mut cmd = Command::cargo_bin("grant").unwrap();
    cmd.arg("apply")
        .arg("--file")
        .arg(path)
        .arg("--dryrun")
        .assert()
        .success()
        .stderr(predicate::str::contains("Summary"))
        .stderr(predicate::str::contains("postgres"))
        .stderr(predicate::str::contains("duyet"))
        .stderr(predicate::str::contains("duyet2"));
}

/// Test `grant validate --file <file>` with invalid role (type: schema)
#[test]
fn validate_file_invalid_role_type_schema() {
    // Create a temporary file with invalid yaml
    let _text = indoc! {"
         connection:
           type: postgres
           url: postgres://postgres:postgres@localhost:5432/postgres
         roles:
         - type: schema
           name: role_schema_level
           grants:
           - invalid
           schemas:
           - schema1
           - schema2
           - schema3
         users: []
    "};

    let mut file = NamedTempFile::new().expect("failed to create temp file");
    file.write(_text.as_bytes())
        .expect("failed to write to temp file");
    let path = PathBuf::from(file.path().to_str().unwrap());

    let mut cmd = Command::cargo_bin("grant").unwrap();
    cmd.arg("apply")
        .arg("--file")
        .arg(path)
        .assert()
        .stderr(predicate::str::contains("invalid grant: invalid"));
}

/// Test `grant validate --file <file>` with invalid role (type: database)
#[test]
fn validate_file_invalid_role_type_database() {
    // Create a temporary file with invalid yaml
    let _text = indoc! {"
         connection:
           type: postgres
           url: postgres://postgres:postgres@localhost:5432/postgres
         roles:
         - type: database
           name: role_database_level
           grants:
           - invalid
           databases:
           - database1
           - database2
           - database3
         users: []
    "};

    let mut file = NamedTempFile::new().expect("failed to create temp file");
    file.write(_text.as_bytes())
        .expect("failed to write to temp file");
    let path = PathBuf::from(file.path().to_str().unwrap());

    let mut cmd = Command::cargo_bin("grant").unwrap();
    cmd.arg("apply")
        .arg("--file")
        .arg(path)
        .assert()
        .stderr(predicate::str::contains("invalid grant: invalid"));
}

/// Test `grant validate --file <file>` with invalid role (type: table)
#[test]
fn validate_file_invalid_role_type_table() {
    // Create a temporary file with invalid yaml
    let _text = indoc! {"
         connection:
           type: postgres
           url: postgres://postgres:postgres@localhost:5432/postgres
         roles:
         - type: table
           name: role_table_level
           grants:
           - invalid
           schemas:
           - schema1
           tables:
           - table1
           - table2
           - table3
         users: []
    "};
    let mut file = NamedTempFile::new().expect("failed to create temp file");
    file.write(_text.as_bytes())
        .expect("failed to write to temp file");
    let path = PathBuf::from(file.path().to_str().unwrap());

    // Validate the file
    let mut cmd = Command::cargo_bin("grant").unwrap();
    cmd.arg("apply")
        .arg("--file")
        .arg(path)
        .assert()
        .stderr(predicate::str::contains("role.grants invalid: invalid"));
}

/// Test `grant validate --file <file>` with invalid role (type: table), missing schemas
#[test]
fn validate_file_invalid_role_type_table_missing_schemas() {
    // Create a temporary file with invalid yaml
    let _text = indoc! {"
         connection:
           type: postgres
           url: postgres://postgres:postgres@localhost:5432/postgres
         roles:
         - type: table
           name: role_table_level
           grants:
           - invalid
           tables:
           - table1
           - table2
           - table3
         users: []
    "};

    let mut file = NamedTempFile::new().expect("failed to create temp file");
    file.write(_text.as_bytes())
        .expect("failed to write to temp file");
    let path = PathBuf::from(file.path().to_str().unwrap());

    // Validate the file
    let mut cmd = Command::cargo_bin("grant").unwrap();
    cmd.arg("apply")
        .arg("--file")
        .arg(path)
        .assert()
        .stderr(predicate::str::contains("roles: missing field `schemas`"));
}

/// Test `grant validate --file <file>` with invalid role (type: table), missing tables
#[test]
fn validate_file_invalid_role_type_table_missing_tables() {
    // Create a temporary file with invalid yaml
    let _text = indoc! {"
         connection:
           type: postgres
           url: postgres://postgres:postgres@localhost:5432/postgres
         roles:
         - type: table
           name: role_table_level
           grants:
           - invalid
           schemas:
           - schema1
         users: []
    "};

    let mut file = NamedTempFile::new().expect("failed to create temp file");
    file.write(_text.as_bytes())
        .expect("failed to write to temp file");
    let path = PathBuf::from(file.path().to_str().unwrap());

    let mut cmd = Command::cargo_bin("grant").unwrap();
    cmd.arg("apply")
        .arg("--file")
        .arg(path)
        .assert()
        .stderr(predicate::str::contains("roles: missing field `tables`"));
}

/// Test `grant validate --file <file>` with invalid role type
#[test]
fn validate_file_invalid_role_type() {
    // Create a temporary file with invalid yaml
    let _text = indoc! {"
         connection:
           type: postgres
           url: postgres://postgres:postgres@localhost:5432/postgres
         roles:
         - type: invalid
           name: role_invalid
           grants:
           - invalid
           schemas:
           - schema1
           - schema2
           - schema3
         users: []
    "};

    let mut file = NamedTempFile::new().expect("failed to create temp file");
    file.write(_text.as_bytes())
        .expect("failed to write to temp file");
    let path = PathBuf::from(file.path().to_str().unwrap());

    let mut cmd = Command::cargo_bin("grant").unwrap();
    cmd.arg("apply")
        .arg("--file")
        .arg(path)
        .assert()
        .stderr(predicate::str::contains(
            "roles[0].type: unknown variant `invalid`",
        ));
}

/// Test `grant validate --file <file>` with user role not existing
#[test]
fn validate_file_user_role_not_existing() {
    // Create a temporary file with invalid yaml
    let _text = indoc! {"
         connection:
           type: postgres
           url: postgres://postgres:postgres@localhost:5432/postgres
         roles:
         - type: table
           name: role_table_1
           schemas:
              - schema1
           grants:
              - SELECT
              - INSERT
              - UPDATE
              - DELETE
           tables:
              - table1
              - table2
              - table3
         users:
            - name: user1
              password: omg
              roles:
              - role_not_existinggggggggggggggg
    "};

    let mut file = NamedTempFile::new().expect("failed to create temp file");
    file.write(_text.as_bytes())
        .expect("failed to write to temp file");
    let path = PathBuf::from(file.path().to_str().unwrap());

    let mut cmd = Command::cargo_bin("grant").unwrap();
    cmd.arg("apply")
        .arg("--file")
        .arg(path)
        .assert()
        .stderr(predicate::str::contains(
            "user role role_not_existinggggggggggggggg is not available",
        ));
}
