use assert_cmd::prelude::*; // Add methods on commands
use predicates::prelude::*; // Used for writing assertions
use std::process::Command; // Run programs

#[test]
fn missing_arguments() {
    let mut cmd = Command::main_binary().unwrap();
    cmd.assert().failure();
}

#[test]
/// `./grant gen` without any args can generate project in current folder
fn gen_without_any_args() {
    let mut cmd = Command::main_binary().unwrap();
    cmd.arg("gen")
        .assert()
        .success()
        .stderr(predicate::str::contains("Generated"));
}

#[test]
fn gen_with_target_args() {
    let mut cmd = Command::main_binary().unwrap();
    cmd.arg("gen")
        .arg("--target")
        .arg("/tmp")
        .assert()
        .success()
        .stderr(predicate::str::contains("/tmp"));
}

#[test]
/// Test gen-pass
fn gen_pass() {
    let mut cmd = Command::main_binary().unwrap();
    cmd.arg("gen-pass")
        .assert()
        .success()
        .stdout(predicate::str::contains("Generated"));
}
