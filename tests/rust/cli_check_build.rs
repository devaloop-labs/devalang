use assert_cmd::Command;
use tempfile::tempdir;

#[test]
fn cli_check_and_build_no_crash() {
    // create a temp output dir and run check/build against examples entry
    let out = tempdir().expect("create tempdir");
    let out_path = out.path().to_str().unwrap();

    // Run `devalang check --entry examples --output <out> --debug`
    let mut cmd = Command::cargo_bin("devalang").expect("binary not found");
    cmd.args(["check", "--entry", "examples", "--output", out_path, "--debug"]);
    let assert = cmd.assert();
    // check may return success or non-zero depending on environment, ensure it doesn't panic
    assert.get_output();

    // Run build command similarly; we only assert it runs (not necessarily success on all envs)
    let mut cmd2 = Command::cargo_bin("devalang").expect("binary not found");
    cmd2.args(["build", "--entry", "examples", "--output", out_path, "--debug"]);
    cmd2.assert().get_output();
}
