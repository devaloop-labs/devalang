use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn cli_prints_help() {
    // Build the binary in debug mode and run it with --help
    let mut cmd = Command::cargo_bin("devalang").expect("binary not found");
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("devalang"));
}
