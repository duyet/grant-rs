use assert_cmd::prelude::*; // Add methods on commands
use predicates::prelude::*; // Used for writing assertions
use std::process::Command; // Run programs

#[test]
fn missing_arguments() {
    let mut cmd = Command::main_binary().unwrap();
    cmd.assert().failure();
}

#[test]
/// `./grant apply` must have --file or -f args
fn apply_missing_arguments() {
    let mut cmd = Command::main_binary().unwrap();
    cmd.arg("apply")
        .assert()
        .failure()
        .stderr(predicate::str::contains("--file"));
}

#[test]
/// `./grant gen` without any args can generate project in current folder
fn gen_without_any_args() {
    let mut cmd = Command::main_binary().unwrap();
    cmd.arg("gen")
        .assert()
        .success()
        .stdout(predicate::str::contains("Generated"));
}

#[test]
fn gen_target_args() {
    let mut cmd = Command::main_binary().unwrap();
    cmd.arg("gen")
        .arg("--target")
        .arg("/tmp")
        .assert()
        .success()
        .stdout(predicate::str::contains("/tmp"));
}
