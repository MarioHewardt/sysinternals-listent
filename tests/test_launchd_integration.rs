use std::process::Command;
use assert_cmd::prelude::*;
use predicates::prelude::*;

#[test]
fn test_launchd_flag_requires_daemon_mode() {
    let mut cmd = Command::cargo_bin("listent").unwrap();
    cmd.args(["--launchd"]);
    
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("--launchd flag requires --daemon mode"));
}

#[test]
fn test_launchd_permission_check() {
    let mut cmd = Command::cargo_bin("listent").unwrap();
    cmd.args(["--daemon", "--launchd"]);
    
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Try running with appropriate permissions"));
}

#[test]
fn test_launchd_help_message() {
    let mut cmd = Command::cargo_bin("listent").unwrap();
    cmd.args(["--help"]);
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("--launchd"))
        .stdout(predicate::str::contains("Install as LaunchD service (requires --daemon and sudo)"));
}

#[test]
fn test_launchd_with_custom_arguments() {
    let mut cmd = Command::cargo_bin("listent").unwrap();
    cmd.args(["--daemon", "--launchd", "--interval", "5.0", "/usr/bin", "-e", "com.test.*"]);
    
    cmd.assert()
        .failure()  // Will fail due to permissions, but should show our custom args
        .stdout(predicate::str::contains("Interval: 5s"))
        .stdout(predicate::str::contains("Paths: [\"/usr/bin\"]"))
        .stdout(predicate::str::contains("Entitlements: [\"com.test.*\"]"));
}

#[test]
fn test_launchd_installation_output() {
    let mut cmd = Command::cargo_bin("listent").unwrap();
    cmd.args(["--daemon", "--launchd"]);
    
    cmd.assert()
        .failure()
        .stdout(predicate::str::contains("🔧 Installing listent as LaunchD service..."));
}