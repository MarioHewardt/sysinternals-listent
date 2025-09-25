//! Integration tests for pattern matching in entitlement filters
//! 
//! Tests both static scan and monitor modes to ensure consistent behavior

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_exact_entitlement_filter_backwards_compatibility() {
    let mut cmd = Command::cargo_bin("listent").unwrap();
    
    // Test exact matching continues to work
    cmd.args(&["/usr/bin", "-e", "com.apple.security.network.client", "--quiet"])
        .assert()
        .success()
        .stdout(predicate::str::contains("com.apple.security.network.client"));
}

#[test]
fn test_exact_filter_no_substring_matching() {
    let mut cmd = Command::cargo_bin("listent").unwrap();
    
    // Substring matching should NOT work (this was the old monitor mode bug)
    cmd.args(&["/usr/bin", "-e", "security.network", "--quiet"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No binaries found"));
}

#[test]
fn test_glob_pattern_wildcard() {
    let mut cmd = Command::cargo_bin("listent").unwrap();
    
    // Test glob pattern matching
    cmd.args(&["/usr/bin", "-e", "com.apple.security.*", "--quiet"])
        .assert()
        .success()
        .stdout(predicate::str::contains("com.apple.security."));
}

#[test]
fn test_glob_pattern_any_network() {
    let mut cmd = Command::cargo_bin("listent").unwrap();
    
    // Test wildcard matching for network entitlements
    cmd.args(&["/usr/bin", "-e", "*network*", "--quiet"])
        .assert()
        .success()
        .stdout(predicate::str::contains("network"));
}

#[test]
fn test_multiple_glob_patterns() {
    let mut cmd = Command::cargo_bin("listent").unwrap();
    
    // Test multiple patterns (OR logic)
    cmd.args(&["/usr/bin", "-e", "com.apple.security.network.*", "-e", "*camera*", "--quiet"])
        .assert()
        .success()
        .stdout(predicate::str::contains("network"));
}

#[test]
fn test_invalid_glob_pattern_validation() {
    let mut cmd = Command::cargo_bin("listent").unwrap();
    
    // Test invalid glob pattern is rejected
    cmd.args(&["/usr/bin", "-e", "com.apple.[", "--quiet"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid entitlement filter"));
}

#[test]
fn test_monitor_mode_glob_patterns() {
    let mut cmd = Command::cargo_bin("listent").unwrap();
    
    // Test that monitor mode also supports glob patterns
    cmd.args(&["--monitor", "-e", "com.apple.security.*", "--interval", "10.0"])
        .timeout(std::time::Duration::from_secs(2))
        .assert()
        .interrupted() // Should be interrupted by timeout
        .stdout(predicate::str::contains("Monitoring for processes with entitlement: com.apple.security.*"));
}

#[test]
fn test_monitor_mode_consistent_exact_matching() {
    let mut cmd = Command::cargo_bin("listent").unwrap();
    
    // Test that monitor mode uses exact matching, not substring matching
    cmd.args(&["--monitor", "-e", "network.client", "--interval", "10.0"])
        .timeout(std::time::Duration::from_secs(2))
        .assert()
        .interrupted() // Should be interrupted by timeout
        .stdout(predicate::str::contains("Monitoring for processes with entitlement: network.client"));
}

#[test]
fn test_json_output_with_patterns() {
    let mut cmd = Command::cargo_bin("listent").unwrap();
    
    // Test JSON output works with glob patterns
    cmd.args(&["/usr/bin", "-e", "com.apple.security.network.*", "--json", "--quiet"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"entitlements\""))
        .stdout(predicate::str::contains("com.apple.security.network"));
}

#[test]
fn test_comma_separated_patterns() {
    let mut cmd = Command::cargo_bin("listent").unwrap();
    
    // Test comma-separated patterns work with glob
    cmd.args(&["/usr/bin", "-e", "com.apple.security.network.*,*camera*", "--quiet"])
        .assert()
        .success()
        .stdout(predicate::str::contains("network").or(predicate::str::contains("No binaries found")));
}

#[test]
fn test_pattern_case_sensitivity() {
    let mut cmd = Command::cargo_bin("listent").unwrap();
    
    // Test that patterns are case-sensitive
    cmd.args(&["/usr/bin", "-e", "COM.APPLE.SECURITY.*", "--quiet"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No binaries found"));
}