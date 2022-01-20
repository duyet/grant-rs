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

/// `grant apply` with target is a directory
#[test]
fn apply_target_is_directory() {
    // create a temporary directory
    let dir = tempfile::tempdir().unwrap();

    let mut cmd = Command::cargo_bin("grant").unwrap();
    cmd.arg("apply")
        .arg("--file")
        .arg(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("directory is not supported"));

    // cleanup
    dir.close().unwrap();
}

/// `grant apply` with target is a directory but --all is set
#[test]
fn apply_target_is_directory_with_all() {
    let test_dir = tempfile::tempdir().unwrap();

    let mut cmd = Command::cargo_bin("grant").unwrap();
    cmd.arg("apply")
        .arg("--file")
        .arg(test_dir.path())
        .arg("--all")
        .assert()
        .success();
}

/// `grant apply` with a config file from `./examples/`
#[test]
fn test_apply_with_config_file_from_example() {
    // read the content from ./examples/example.yaml
    let text = std::fs::read_to_string("./examples/example.yaml").unwrap();
    let mut file = NamedTempFile::new().expect("failed to create temp file");
    file.write(text.as_bytes())
        .expect("failed to write to temp file");
    let path = PathBuf::from(file.path().to_str().unwrap());

    // Command `grant apply` with --file
    let mut cmd = Command::cargo_bin("grant").unwrap();
    let apply = cmd.arg("apply").arg("--file").arg(path).arg("--dryrun");

    apply.assert().success();

    apply.assert().stderr(predicate::str::contains(
        "Connected to database: postgres://postgres:postgres@localhost:5432/postgres",
    ));

    let expected = indoc! {r#"
        ┌────────┬─────────────────────┬──────────────────────┬─────────┐
        │ User   │ Role Name           │ Detail               │ Status  │
        │ ---    │ ---                 │ ---                  │ ---     │
        │ duyet  │ role_database_level │ database["postgres"] │ dry-run │
        │ duyet  │ role_schema_level   │ schema["public"]     │ dry-run │
        │ duyet  │ role_table_level    │ table["ALL"]         │ dry-run │
        │ duyet2 │ role_database_level │ database["postgres"] │ dry-run │
        │ duyet2 │ role_schema_level   │ schema["public"]     │ dry-run │
        │ duyet2 │ role_table_level    │ table["ALL"]         │ dry-run │
        │ duyet3 │ role_database_level │ database["postgres"] │ dry-run │
        │ duyet3 │ role_schema_level   │ schema["public"]     │ dry-run │
        │ duyet3 │ role_table_level    │ table["ALL"]         │ dry-run │
        └────────┴─────────────────────┴──────────────────────┴─────────┘
    "#};

    // Test output of grant
    for line in expected.lines() {
        apply.assert().stderr(predicate::str::contains(line));
    }

    // TODO: test output of create users
}
