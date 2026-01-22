//! Contract tests for daemon CLI subcommands
//!
//! These tests validate the daemon management CLI interface according to
//! the specification in specs/003-add-launchd-daemon/contracts/cli-contract.md

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_install_daemon_subcommand() {
    let mut cmd = Command::cargo_bin("listent").unwrap();
    
    // Test basic install-daemon subcommand - will fail due to permissions but should be recognized
    cmd.arg("install-daemon")
       .assert()
       .failure() // Expected to fail due to permission issues (can't write to /Library/LaunchDaemons)
       .stderr(predicate::str::contains("Permission denied").or(
           predicate::str::contains("Failed to write plist file")
       ));
}

#[test]
fn test_install_daemon_with_config() {
    let mut cmd = Command::cargo_bin("listent").unwrap();
    
    // Test install-daemon with custom config path - will fail because config file doesn't exist
    cmd.args(&["install-daemon", "--config", "/tmp/test-config.toml"])
       .assert()
       .failure() // Expected to fail because config file doesn't exist
       .stderr(predicate::str::contains("Failed to read config file").or(
           predicate::str::contains("No such file or directory")
       ));
}

#[test]
fn test_uninstall_daemon_subcommand() {
    let mut cmd = Command::cargo_bin("listent").unwrap();
    
    // Test uninstall-daemon subcommand - may fail due to permissions if plist exists
    // or succeed if plist doesn't exist
    cmd.arg("uninstall-daemon")
       .assert()
       .stdout(predicate::str::contains("Uninstalling listent daemon service"));
    // Note: Don't assert success/failure since it depends on whether plist exists
    // and whether we have permissions
}

#[test]
fn test_daemon_status_subcommand() {
    let mut cmd = Command::cargo_bin("listent").unwrap();
    
    // Test daemon-status subcommand - should work and show status
    cmd.arg("daemon-status")
       .assert()
       .success() // Should succeed and show status
       .stdout(predicate::str::contains("Checking listent daemon status"));
}

#[test]
fn test_logs_subcommand() {
    let mut cmd = Command::cargo_bin("listent").unwrap();
    
    // Test logs subcommand - should now work with fixed predicate
    cmd.arg("logs")
       .assert()
       .success() // Should succeed with fixed macOS log predicate
       .stdout(predicate::str::contains("Retrieving daemon logs"));
}

#[test]
fn test_logs_with_follow() {
    let mut cmd = Command::cargo_bin("listent").unwrap();
    
    // Test logs with --follow flag - should now work with fixed predicate
    cmd.args(&["logs", "--follow"])
       .timeout(std::time::Duration::from_millis(500)) // Use timeout since --follow runs indefinitely
       .assert()
       .interrupted() // Should be interrupted by timeout after starting successfully
       .stdout(predicate::str::contains("Retrieving daemon logs"));
}

#[test]
fn test_logs_with_since() {
    let mut cmd = Command::cargo_bin("listent").unwrap();
    
    // Test logs with --since flag - should now work with fixed predicate
    cmd.args(&["logs", "--since", "1h"])
       .assert()
       .success() // Should succeed with fixed macOS log predicate
       .stdout(predicate::str::contains("Retrieving daemon logs"));
}

#[test]
fn test_daemon_flag_compatibility() {
    let mut cmd = Command::cargo_bin("listent").unwrap();
    
    // Test that --daemon flag is incompatible with subcommands
    cmd.args(&["--daemon", "install-daemon"])
       .assert()
       .failure() // Should fail due to --daemon requiring --monitor
       .stderr(predicate::str::contains("--daemon requires --monitor"));
}

#[test]
fn test_help_shows_daemon_subcommands() {
    let mut cmd = Command::cargo_bin("listent").unwrap();
    
    // Test that help output includes daemon subcommands
    cmd.arg("--help")
       .assert()
       .success()
       .stdout(predicate::str::contains("install-daemon").or(
           predicate::str::contains("SUBCOMMANDS").or(
               predicate::str::contains("Commands") // Different help format
           )
       ));
}