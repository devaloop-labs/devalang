use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn cli_prints_version() {
    let mut cmd = Command::cargo_bin("devalang").expect("binary not found");
    cmd.arg("--version");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("devalang"));
}
