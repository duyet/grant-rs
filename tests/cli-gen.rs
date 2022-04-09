use assert_cmd::prelude::*; // Add methods on commands
use predicates::prelude::*; // Used for writing assertions
use std::process::Command; // Run programs

#[test]
fn missing_arguments() {
    let mut cmd = Command::cargo_bin("grant").unwrap();
    cmd.assert().failure();
}

#[test]
/// `./grant gen` without any args can generate project in current folder
fn gen_without_any_args() {
    let mut cmd = Command::cargo_bin("grant").unwrap();
    cmd.arg("gen")
        .assert()
        .success()
        // because run test in current folder
        .stderr(predicate::str::contains("target already exists"));
}

#[test]
fn gen_with_target_args() {
    // Random folder name in /tmp
    let folder_name = format!("/tmp/{}", rand::random::<u64>());

    let mut cmd = Command::cargo_bin("grant").unwrap();
    cmd.arg("gen")
        .arg("--target")
        .arg(folder_name.clone())
        .assert()
        .success()
        .stderr(predicate::str::contains("Generated"))
        .stderr(predicate::str::contains(folder_name));
}

#[test]
/// Test gen-pass
fn gen_pass() {
    let mut cmd = Command::cargo_bin("grant").unwrap();
    cmd.arg("gen-pass")
        .assert()
        .success()
        .stdout(predicate::str::contains("Generated password:"))
        .stdout(predicate::str::contains(
            "Hint: Please provide --username to generate MD5",
        ));
}

#[test]
/// Test gen-pass with --username
fn gen_pass_with_username() {
    let mut cmd = Command::cargo_bin("grant").unwrap();
    cmd.arg("gen-pass")
        .arg("--username")
        .arg("test")
        .assert()
        .success()
        .stdout(predicate::str::contains("Generated password:"))
        .stdout(predicate::str::contains("Generated MD5"));
}

#[test]
/// Test gen-pass with --username and --password, generate MD5
/// For example:
/// ```
/// ./grant gen-pass --username duyet --password 123456
///
/// Generated password: 123456
/// Generated MD5 (user: duyet): md5de3331387913465470ce1772a279be8e
/// ```
fn gen_pass_with_username_and_password() {
    let mut cmd = Command::cargo_bin("grant").unwrap();
    cmd.arg("gen-pass")
        .arg("--username")
        .arg("duyet")
        .arg("--password")
        .arg("123456")
        .assert()
        .success()
        .stdout(predicate::str::contains("Generated password:"))
        .stdout(predicate::str::contains("Generated MD5 (user: duyet):"));
}
