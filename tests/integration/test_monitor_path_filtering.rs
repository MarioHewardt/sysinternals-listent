use assert_cmd::Command;
use predicates::prelude::*;
use std::time::Duration;

#[test]
fn test_monitor_with_single_path_filter() {
    let mut cmd = Command::cargo_bin("listent").unwrap();
    cmd.args(&["--monitor", "-p", "/Applications", "--interval", "1.0"])
        .timeout(Duration::from_secs(3))
        .assert()
        .success()
        .stdout(predicate::str::contains("Monitoring /Applications"));
}

#[test]
fn test_monitor_with_multiple_path_filters() {
    let mut cmd = Command::cargo_bin("listent").unwrap();
    cmd.args(&[
        "--monitor", 
        "-p", "/Applications",
        "-p", "/System/Applications",
        "--interval", "1.0"
    ])
    .timeout(Duration::from_secs(3))
    .assert()
    .success()
    .stdout(predicate::str::contains("Monitoring"));
}

#[test]
fn test_monitor_with_nonexistent_path() {
    let mut cmd = Command::cargo_bin("listent").unwrap();
    cmd.args(&["--monitor", "-p", "/nonexistent/path", "--interval", "1.0"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("does not exist").or(
            predicate::str::contains("not a directory")
        ));
}

#[test]
fn test_path_filtering_effectiveness() {
    // Test that path filtering actually works by comparing output
    // This test runs monitor with specific path filter and validates
    // that startup messages reflect the filtering
    let mut cmd = Command::cargo_bin("listent").unwrap();
    cmd.args(&["--monitor", "-p", "/usr/bin", "--interval", "1.0"])
        .timeout(Duration::from_secs(2))
        .assert()
        .success()
        .stdout(predicate::str::contains("/usr/bin"));
}

#[test]
fn test_monitor_system_applications_path() {
    let mut cmd = Command::cargo_bin("listent").unwrap();
    cmd.args(&["--monitor", "-p", "/System/Applications", "--interval", "1.0"])
        .timeout(Duration::from_secs(2))
        .assert()
        .success()
        .stdout(predicate::str::contains("/System/Applications"));
}

#[test]
fn test_path_filter_with_json_output() {
    let mut cmd = Command::cargo_bin("listent").unwrap();
    cmd.args(&[
        "--monitor", 
        "-p", "/Applications",
        "--json",
        "--interval", "1.0"
    ])
    .timeout(Duration::from_secs(2))
    .assert()
    .success(); // Should output JSON format with path filtering
}

#[test]
fn test_path_filter_validation() {
    // Test various invalid path scenarios
    let mut cmd = Command::cargo_bin("listent").unwrap();
    cmd.args(&["--monitor", "-p", "", "--interval", "1.0"])
        .assert()
        .failure();
        
    let mut cmd = Command::cargo_bin("listent").unwrap();
    cmd.args(&["--monitor", "-p", "/dev/null", "--interval", "1.0"])
        .assert()
        .failure(); // /dev/null is not a directory
}