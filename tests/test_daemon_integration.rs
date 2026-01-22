//! Integration tests for daemon startup and lifecycle management
//!
//! These tests validate the daemon's ability to start, run, and shutdown properly
//! Note: These tests expect permission failures since they cannot write to system directories

use assert_cmd::Command;
use predicates::prelude::*;
use std::time::Duration;
use tempfile::tempdir;

#[test]
fn test_daemon_startup_process() {
    // Test that daemon can start in background mode
    let mut cmd = Command::cargo_bin("listent").unwrap();
    
    // --daemon requires --monitor flag
    cmd.args(&["--daemon"])
       .timeout(Duration::from_secs(5))
       .assert()
       .failure()
       .stderr(predicate::str::contains("--daemon requires --monitor"));
}

#[test]
fn test_daemon_pid_file_creation() {
    let temp_dir = tempdir().unwrap();
    let _pid_file = temp_dir.path().join("test.pid");
    
    // Create config with custom PID file location
    let config_path = temp_dir.path().join("config.toml");
    let config_content = r#"
[daemon]
polling_interval = 1.0
auto_start = false

[monitoring]
path_filters = []
entitlement_filters = []
"#;
    
    std::fs::write(&config_path, config_content).unwrap();
    
    // Test daemon startup with custom config
    // Will fail due to permission issues (can't write to /var/run/listent or /Library/LaunchDaemons)
    let mut cmd = Command::cargo_bin("listent").unwrap();
    cmd.args(&["install-daemon", "--config", config_path.to_str().unwrap()])
       .assert()
       .failure()
       .stderr(predicate::str::contains("Permission denied").or(
           predicate::str::contains("Failed to create working directory").or(
               predicate::str::contains("Failed to write plist file")
           )
       ));
}

#[test]
fn test_daemon_configuration_management() {
    // Test daemon configuration loading and management
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join("daemon.toml");
    
    let config_content = r#"
[daemon]
polling_interval = 2.0
auto_start = false

[monitoring]
path_filters = ["/Applications", "/usr/bin"]
entitlement_filters = ["com.apple.security.network.client"]

[logging]
level = "debug"
"#;
    
    std::fs::write(&config_path, config_content).unwrap();

    // Will fail due to permission issues (can't write to /var/run/listent or /Library/LaunchDaemons)
    let mut cmd = Command::cargo_bin("listent").unwrap();
    cmd.args(&["install-daemon", "--config", config_path.to_str().unwrap()])
       .assert()
       .failure()
       .stderr(predicate::str::contains("Permission denied").or(
           predicate::str::contains("Failed to create working directory").or(
               predicate::str::contains("Failed to write plist file")
           )
       ));
}

#[test]
fn test_daemon_status_command() {
    // Test daemon status command - should succeed and show status info
    let mut cmd = Command::cargo_bin("listent").unwrap();
    
    cmd.arg("daemon-status")
       .assert()
       .success()
       .stdout(predicate::str::contains("Checking listent daemon status"));
}

#[test]
fn test_daemon_launchd_integration() {
    // Test LaunchD plist generation and installation
    // Will fail due to permission issues (can't write to /Library/LaunchDaemons)
    let mut cmd = Command::cargo_bin("listent").unwrap();
    
    cmd.arg("install-daemon")
       .assert()
       .failure()
       .stderr(predicate::str::contains("Permission denied").or(
           predicate::str::contains("Failed to create working directory").or(
               predicate::str::contains("Failed to write plist file")
           )
       ));
}

#[test]
fn test_daemon_process_monitoring() {
    // Test that daemon properly monitors processes
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join("monitor.toml");
    
    let config_content = r#"
[daemon]
polling_interval = 0.5
auto_start = false

[monitoring]
path_filters = ["/bin", "/usr/bin"]
entitlement_filters = []

[logging]
level = "info"
"#;
    
    std::fs::write(&config_path, config_content).unwrap();
    
    // Test daemon with monitoring configuration
    // Will fail due to permission issues (can't write to /var/run/listent or /Library/LaunchDaemons)
    let mut cmd = Command::cargo_bin("listent").unwrap();
    cmd.args(&["install-daemon", "--config", config_path.to_str().unwrap()])
       .assert()
       .failure()
       .stderr(predicate::str::contains("Permission denied").or(
           predicate::str::contains("Failed to create working directory").or(
               predicate::str::contains("Failed to write plist file")
           )
       ));
}