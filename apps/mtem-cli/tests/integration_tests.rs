use assert_cmd::Command;

#[test]
fn test_list_command() {
    let mut cmd = Command::cargo_bin("mtem").unwrap();
    cmd.arg("list");

    cmd.assert()
        .success()
        .stdout(predicates::str::contains("Available templates"));
}

#[test]
fn test_help_command() {
    let mut cmd = Command::cargo_bin("mtem").unwrap();
    cmd.arg("--help");

    cmd.assert()
        .success()
        .stdout(predicates::str::contains("Masstemplate"));
}

#[test]
fn test_info_command_nonexistent_template() {
    let mut cmd = Command::cargo_bin("mtem").unwrap();
    cmd.arg("info").arg("nonexistent");

    cmd.assert()
        .failure()
        .stderr(predicates::str::contains("Template 'nonexistent' not found"));
}