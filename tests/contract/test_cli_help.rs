use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_help_includes_required_options() {
    let mut cmd = Command::cargo_bin("listent").unwrap();
    cmd.arg("--help");
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("--entitlement"))
        .stdout(predicate::str::contains("--json"))
        .stdout(predicate::str::contains("--quiet"))
        .stdout(predicate::str::contains("--monitor"))
        .stdout(predicate::str::contains("--interval"))
        .stdout(predicate::str::contains("--daemon"))
        .stdout(predicate::str::contains("--launchd"))
        .stdout(predicate::str::contains("--help"))
        .stdout(predicate::str::contains("--version"));
}

#[test]
fn test_help_describes_path_option() {
    let mut cmd = Command::cargo_bin("listent").unwrap();
    cmd.arg("--help");
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("directory"))
        .stdout(predicate::str::contains("path"));
}

#[test]
fn test_help_describes_entitlement_filter() {
    let mut cmd = Command::cargo_bin("listent").unwrap();
    cmd.arg("--help");
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("entitlement"))
        .stdout(predicate::str::contains("filter"));
}