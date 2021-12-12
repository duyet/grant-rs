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
        .stderr(predicate::str::contains("is a directory"));

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

    let mut cmd = Command::cargo_bin("grant").unwrap();
    let apply = cmd.arg("apply").arg("--file").arg(path);

    apply.assert().success();

    apply.assert().stderr(predicate::str::contains(
        "Connected to database: postgres://postgres:postgres@localhost:5432/postgres",
    ));

    let expected = indoc! {r#"
        │ User     │ Action                     │
        │ ---      │ ---                        │
        │ duyet    │ no action (already exists) │
        │ duyet2   │ no action (already exists) │
        │ duyet3   │ no action (already exists) │
        ┌────────┬─────────────────────┬──────────────────────┬────────┬
        │ User   │ Role Name           │ Detail               │ Action │
        │ ---    │ ---                 │ ---                  │ ---    │
        │ duyet  │ role_database_level │ database["postgres"] │ grant  │
        │ duyet  │ role_all_schema     │ table["ALL"]         │ grant  │
        │ duyet  │ role_schema_level   │ schema["public"]     │ grant  │
        │ duyet2 │ role_database_level │ database["postgres"] │ grant  │
        │ duyet2 │ role_all_schema     │ table["ALL"]         │ grant  │
        │ duyet2 │ role_schema_level   │ schema["public"]     │ grant  │
        │ duyet3 │ role_database_level │ database["postgres"] │ grant  │
        │ duyet3 │ role_all_schema     │ table["ALL"]         │ grant  │
        │ duyet3 │ -role_schema_level  │ schema["public"]     │ revoke │
        └────────┴─────────────────────┴──────────────────────┴────────┴
    "#};

    for line in expected.lines() {
        apply.assert().stderr(predicate::str::contains(line));
    }
}
