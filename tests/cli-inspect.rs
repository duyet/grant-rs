use assert_cmd::prelude::*; // Add methods on commands
use indoc::indoc;
use predicates::prelude::*; // Used for writing assertions
use std::io::Write;
use std::path::PathBuf;
use std::process::Command; // Run programs
use tempfile::NamedTempFile;

#[test]
/// grant inspect should print the current user
fn inspect_grant() {
    // Create a temporary file with invalid yaml
    let _text = indoc! {"
         connection:
           type: postgres
           url: postgres://postgres@localhost:5432/postgres
         roles: []
         users: []
    "};

    let mut file = NamedTempFile::new().expect("failed to create temp file");
    file.write(_text.as_bytes())
        .expect("failed to write to temp file");
    let path = PathBuf::from(file.path().to_str().unwrap());

    let mut cmd = Command::cargo_bin("grant").unwrap();
    cmd.arg("inspect").arg("--file").arg(path);

    // Expect output contains the current user name
    cmd.assert()
        .success()
        .stderr(predicate::str::contains("postgres"));
}
