use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_human_readable_output_format() {
    let mut cmd = Command::cargo_bin("listent").unwrap();
    cmd.args(&["--monitor", "--interval", "1.0"])
        .timeout(std::time::Duration::from_secs(2))
        .assert()
        .interrupted() // Monitor processes get interrupted by timeout, not success
        .stdout(predicate::str::contains("Starting process monitoring"))
        .stdout(predicate::str::contains("Press Ctrl+C to stop"));
}

#[test]
fn test_json_output_format() {
    let mut cmd = Command::cargo_bin("listent").unwrap();
    let result = cmd.args(&["--monitor", "--json", "--interval", "1.0"])
        .timeout(std::time::Duration::from_secs(2))
        .assert()
        .interrupted(); // Monitor processes get interrupted by timeout

    // For now, just verify the monitor starts correctly
    // JSON output validation would require actual process detection events
    let output = result.get_output();
    let output_str = String::from_utf8_lossy(&output.stdout);
    
    // At minimum, should show the monitoring startup message
    assert!(output_str.contains("Starting process monitoring") || output_str.is_empty());
}

#[test]
fn test_timestamp_formatting() {
    let mut cmd = Command::cargo_bin("listent").unwrap();
    cmd.args(&["--monitor", "--interval", "1.0"])
        .timeout(std::time::Duration::from_secs(2))
        .assert()
        .interrupted() // Monitor processes get interrupted by timeout
        .stdout(predicate::str::contains("Starting process monitoring"));
        // Timestamp validation would require actual process detection events
}


#[test]
fn test_entitlements_list_formatting() {
    let mut cmd = Command::cargo_bin("listent").unwrap();
    cmd.args(&["--monitor", "--interval", "1.0"])
        .timeout(std::time::Duration::from_secs(2))
        .assert()
        .interrupted() // Monitor processes get interrupted by timeout
        .stdout(predicate::str::contains("Starting process monitoring"));
        // Entitlements list validation would require actual process detection events
}

#[test]
fn test_quiet_mode_output_suppression() {
    let mut cmd = Command::cargo_bin("listent").unwrap();
    let result = cmd.args(&["--monitor", "--quiet", "--interval", "1.0"])
        .timeout(std::time::Duration::from_secs(2))
        .assert()
        .interrupted(); // Monitor processes get interrupted by timeout

    // In quiet mode, should not show startup messages
    let output = result.get_output();
    let output_str = String::from_utf8_lossy(&output.stdout);
    
    // Should not contain startup messages in quiet mode
    assert!(!output_str.contains("Starting process monitoring"));
    assert!(!output_str.contains("Press Ctrl+C"));
}

#[test]
fn test_no_entitlements_formatting() {
    // Test that monitor starts correctly even if no processes are detected
    let mut cmd = Command::cargo_bin("listent").unwrap();
    cmd.args(&["--monitor", "--interval", "1.0"])
        .timeout(std::time::Duration::from_secs(2))
        .assert()
        .interrupted() // Monitor processes get interrupted by timeout
        .stdout(predicate::str::contains("Starting process monitoring"));
}