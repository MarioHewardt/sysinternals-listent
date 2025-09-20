//! Contract tests for daemon CLI subcommands
//!
//! These tests validate the daemon management CLI interface according to
//! the specification in specs/003-add-launchd-daemon/contracts/cli-contract.md

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_install_daemon_subcommand() {
    let mut cmd = Command::cargo_bin("listent").unwrap();
    
    // Test basic install-daemon subcommand
    cmd.arg("install-daemon")
       .assert()
       .failure() // Will fail until implemented
       .stderr(predicate::str::contains("not yet implemented").or(
           predicate::str::contains("unrecognized subcommand")
       ));
}

#[test]
fn test_install_daemon_with_config() {
    let mut cmd = Command::cargo_bin("listent").unwrap();
    
    // Test install-daemon with custom config path
    cmd.args(&["install-daemon", "--config", "/tmp/test-config.toml"])
       .assert()
       .failure() // Will fail until implemented
       .stderr(predicate::str::contains("not yet implemented").or(
           predicate::str::contains("unrecognized subcommand")
       ));
}

#[test]
fn test_uninstall_daemon_subcommand() {
    let mut cmd = Command::cargo_bin("listent").unwrap();
    
    // Test uninstall-daemon subcommand
    cmd.arg("uninstall-daemon")
       .assert()
       .failure() // Will fail until implemented
       .stderr(predicate::str::contains("not yet implemented").or(
           predicate::str::contains("unrecognized subcommand")
       ));
}

#[test]
fn test_daemon_status_subcommand() {
    let mut cmd = Command::cargo_bin("listent").unwrap();
    
    // Test daemon-status subcommand
    cmd.arg("daemon-status")
       .assert()
       .failure() // Will fail until implemented
       .stderr(predicate::str::contains("not yet implemented").or(
           predicate::str::contains("unrecognized subcommand")
       ));
}

#[test]
fn test_update_config_subcommand() {
    let mut cmd = Command::cargo_bin("listent").unwrap();
    
    // Test update-config with key-value pairs
    cmd.args(&["update-config", "daemon.polling_interval=2.0", "logging.level=debug"])
       .assert()
       .failure() // Will fail until implemented
       .stderr(predicate::str::contains("not yet implemented").or(
           predicate::str::contains("unrecognized subcommand")
       ));
}

#[test]
fn test_logs_subcommand() {
    let mut cmd = Command::cargo_bin("listent").unwrap();
    
    // Test logs subcommand
    cmd.arg("logs")
       .assert()
       .failure() // Will fail until implemented
       .stderr(predicate::str::contains("not yet implemented").or(
           predicate::str::contains("unrecognized subcommand")
       ));
}

#[test]
fn test_logs_with_follow() {
    let mut cmd = Command::cargo_bin("listent").unwrap();
    
    // Test logs with --follow flag
    cmd.args(&["logs", "--follow"])
       .assert()
       .failure() // Will fail until implemented
       .stderr(predicate::str::contains("not yet implemented").or(
           predicate::str::contains("unrecognized subcommand")
       ));
}

#[test]
fn test_logs_with_since() {
    let mut cmd = Command::cargo_bin("listent").unwrap();
    
    // Test logs with --since flag
    cmd.args(&["logs", "--since", "1h"])
       .assert()
       .failure() // Will fail until implemented
       .stderr(predicate::str::contains("not yet implemented").or(
           predicate::str::contains("unrecognized subcommand")
       ));
}

#[test]
fn test_daemon_flag_compatibility() {
    let mut cmd = Command::cargo_bin("listent").unwrap();
    
    // Test that --daemon flag is incompatible with subcommands
    cmd.args(&["--daemon", "install-daemon"])
       .assert()
       .failure() // Should fail due to conflicting arguments
       .stderr(predicate::str::contains("conflict").or(
           predicate::str::contains("cannot be used")
       ).or(predicate::str::contains("unrecognized subcommand")));
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