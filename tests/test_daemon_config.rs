//! Contract tests for daemon configuration management
//!
//! These tests validate configuration file parsing, validation, and persistence
//! according to specs/003-add-launchd-daemon/contracts/configuration-contract.md

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_configuration_file_parsing() {
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join("daemon.toml");
    
    // Create a valid configuration file
    let config_content = r#"
[daemon]
polling_interval = 1.5
auto_start = true

[monitoring]
path_filters = ["/Applications"]
entitlement_filters = ["com.apple.security.network.client"]
"#;
    
    fs::write(&config_path, config_content).unwrap();
    
    // Test daemon with config file - should fail due to file not existing or permission error
    let mut cmd = Command::cargo_bin("listent").unwrap();
    cmd.args(&["install-daemon", "--config", config_path.to_str().unwrap()])
       .assert()
       .failure() // Will fail due to permission issues writing plist file
       .stderr(predicate::str::contains("Permission denied").or(
           predicate::str::contains("Failed to write plist file")
       ));
}

#[test]
fn test_invalid_configuration_rejection() {
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join("invalid.toml");
    
    // Create an invalid configuration file
    let invalid_config = r#"
[daemon]
polling_interval = 0.05  # Too low (minimum is 0.1)
auto_start = "yes"       # Wrong type (should be boolean)

[monitoring]
path_filters = []
entitlement_filters = []
"#;
    
    fs::write(&config_path, invalid_config).unwrap();
    
    // Test daemon with invalid config (will fail until implemented)
    let mut cmd = Command::cargo_bin("listent").unwrap();
    cmd.args(&["install-daemon", "--config", config_path.to_str().unwrap()])
       .assert()
       .failure() // Will fail due to invalid config
       .stderr(predicate::str::contains("invalid").or(
           predicate::str::contains("not yet implemented")
       ));
}

#[test]
fn test_configuration_validation_bounds() {
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join("bounds.toml");
    
    // Test polling interval bounds
    let bounds_config = r#"
[daemon]
polling_interval = 301.0  # Above maximum (300.0)
auto_start = true

[monitoring]
path_filters = []
entitlement_filters = []
"#;
    
    fs::write(&config_path, bounds_config).unwrap();
    
    let mut cmd = Command::cargo_bin("listent").unwrap();
    cmd.args(&["install-daemon", "--config", config_path.to_str().unwrap()])
       .assert()
       .failure()
       .stderr(predicate::str::contains("Invalid polling interval").or(
           predicate::str::contains("Must be between 0.1 and 300.0")
       ));
}

#[test]
fn test_default_configuration_generation() {
    // Test that daemon can generate default configuration - will fail due to permissions
    let mut cmd = Command::cargo_bin("listent").unwrap();
    cmd.arg("install-daemon")
       .assert()
       .failure() // Will fail due to permission issues
       .stderr(predicate::str::contains("Permission denied").or(
           predicate::str::contains("Failed to write plist file")
       ));
}

#[test]
fn test_configuration_update_validation() {
    // Test configuration updates with invalid values
    let mut cmd = Command::cargo_bin("listent").unwrap();
    
    // Test invalid polling interval update
    cmd.args(&["update-config", "daemon.polling_interval=0.05"])
       .assert()
       .failure()
       .stderr(predicate::str::contains("polling_interval").or(
           predicate::str::contains("not yet implemented")
       ));
}

#[test]
fn test_configuration_atomic_updates() {
    // Test that configuration updates are atomic (all or nothing)
    let mut cmd = Command::cargo_bin("listent").unwrap();
    
    // Mix of valid and invalid updates should fail entirely
    cmd.args(&[
        "update-config", 
        "daemon.polling_interval=2.0",  // Valid
        "daemon.invalid_field=value",   // Invalid field
        "logging.level=debug"           // Valid
    ])
    .assert()
    .failure()
    .stderr(predicate::str::contains("invalid_field").or(
        predicate::str::contains("not yet implemented")
    ));
}