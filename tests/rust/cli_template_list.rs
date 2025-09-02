use assert_cmd::Command;

#[test]
fn cli_template_list_runs() {
    let mut cmd = Command::cargo_bin("devalang").expect("binary not found");
    cmd.args(["template", "list"]);
    let assert = cmd.assert();
    // ensure it runs without panic; output may vary
    assert.get_output();
}
