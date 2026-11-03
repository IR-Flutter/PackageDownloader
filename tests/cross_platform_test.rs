use assert_cmd::Command;

#[test]
fn test_cli_version_flag() {
    let mut cmd = Command::cargo_bin("pdown").unwrap();
    cmd.arg("--version");
    cmd.assert().success();

    let mut cmd_alias = Command::cargo_bin("irflutter").unwrap();
    cmd_alias.arg("--version");
    cmd_alias.assert().success();
}

#[test]
fn test_cli_list_mirrors() {
    let mut cmd = Command::cargo_bin("pdown").unwrap();
    cmd.arg("list");
    let assert = cmd.assert().success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();

    assert!(stdout.contains("PUB-AZS Community Mirror"));
    assert!(stdout.contains("Runflare Cloud Mirrors"));
    assert!(stdout.contains("Flutter Community Mirror (CN)"));
}

#[test]
fn test_cli_json_mode_validity() {
    let mut cmd = Command::cargo_bin("pdown").unwrap();
    cmd.args(["list", "--json"]);
    let assert = cmd.assert().success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();

    let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("Must be valid JSON");
    assert!(parsed.is_array());
    assert!(!parsed.as_array().unwrap().is_empty());
}

#[test]
fn test_cli_doctor_json_mode() {
    let mut cmd = Command::cargo_bin("pdown").unwrap();
    cmd.args(["doctor", "--json"]);
    let assert = cmd.assert().success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();

    let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("Doctor JSON must parse");
    assert!(parsed.is_array());
}

#[test]
fn test_cli_use_dry_run() {
    let mut cmd = Command::cargo_bin("pdown").unwrap();
    cmd.args(["use", "pub-azs", "--dry-run"]);
    let assert = cmd.assert().success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();

    assert!(stdout.contains("[DRY-RUN]"));
    assert!(stdout.contains("PUB_HOSTED_URL"));
}

#[test]
fn test_cli_alias_irflutter_parity() {
    let mut cmd_pdown = Command::cargo_bin("pdown").unwrap();
    cmd_pdown.args(["list", "--json"]);
    let out_pdown = cmd_pdown.output().unwrap().stdout;

    let mut cmd_irflutter = Command::cargo_bin("irflutter").unwrap();
    cmd_irflutter.args(["list", "--json"]);
    let out_irflutter = cmd_irflutter.output().unwrap().stdout;

    assert_eq!(
        out_pdown, out_irflutter,
        "irflutter and pdown output must match identically"
    );
}
