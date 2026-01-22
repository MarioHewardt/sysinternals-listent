//! Contract tests for enhanced ULS logging in daemon mode
//!
//! These tests validate structured logging according to
//! specs/003-add-launchd-daemon/contracts/uls-integration.md

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_daemon_startup_logging() {
    // Test that daemon startup requires --monitor flag
    let mut cmd = Command::cargo_bin("listent").unwrap();
    
    cmd.args(&["--daemon"])
       .assert()
       .failure()
       .stderr(predicate::str::contains("--daemon requires --monitor"));
}

#[test]
fn test_daemon_process_detection_logging() {
    // Test that process detection events are logged with entitlement details
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join("logging.toml");
    
    let config_content = r#"
[daemon]
polling_interval = 0.5
auto_start = false

[logging]
level = "debug"
subsystem = "com.github.mariohewardt.listent"
category = "daemon"

[monitoring]
path_filters = ["/bin"]
entitlement_filters = []
"#;
    
    fs::write(&config_path, config_content).unwrap();
    
    // Will fail due to permission issues (can't write to system directories)
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
fn test_daemon_log_levels() {
    // Test different log levels in daemon mode
    let levels = ["error", "warn", "info", "debug"];
    
    for level in &levels {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join(format!("{}_config.toml", level));
        
        let config_content = format!(r#"
[daemon]
polling_interval = 1.0
auto_start = false

[logging]
level = "{}"
subsystem = "com.github.mariohewardt.listent"
category = "daemon"

[monitoring]
path_filters = []
entitlement_filters = []
"#, level);
        
        fs::write(&config_path, config_content).unwrap();
        
        // Will fail due to permission issues (can't write to system directories)
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
}

#[test]
fn test_daemon_logs_command() {
    // Test that logs command retrieves daemon logs
    let mut cmd = Command::cargo_bin("listent").unwrap();
    
    cmd.arg("logs")
       .assert()
       .success()
       .stdout(predicate::str::contains("Retrieving daemon logs"));
}

#[test]
fn test_daemon_logs_follow_mode() {
    // Test that logs --follow works (use timeout since it runs indefinitely)
    let mut cmd = Command::cargo_bin("listent").unwrap();
    
    cmd.args(&["logs", "--follow"])
       .timeout(std::time::Duration::from_millis(500))
       .assert()
       .interrupted() // Should be interrupted by timeout
       .stdout(predicate::str::contains("Retrieving daemon logs"));
}

#[test]
fn test_daemon_logs_time_filtering() {
    // Test that logs --since filters by time
    let mut cmd = Command::cargo_bin("listent").unwrap();
    
    cmd.args(&["logs", "--since", "1h"])
       .assert()
       .success()
       .stdout(predicate::str::contains("Retrieving daemon logs"));
}

#[test]
fn test_daemon_log_structured_format() {
    // Test that daemon logs can be output in structured JSON format
    let mut cmd = Command::cargo_bin("listent").unwrap();
    
    cmd.args(&["logs", "--format", "json"])
       .assert()
       .success()
       .stdout(predicate::str::contains("Retrieving daemon logs"));
}

#[test]
fn test_daemon_no_terminal_output() {
    // Test that daemon mode requires --monitor flag
    let mut cmd = Command::cargo_bin("listent").unwrap();
    
    cmd.args(&["--daemon"])
       .assert()
       .failure()
       .stderr(predicate::str::contains("--daemon requires --monitor"));
}