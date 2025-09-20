use assert_cmd::Command;
use predicates::prelude::*;
use std::time::Duration;

#[test]
fn test_monitor_json_output_format() {
    let mut cmd = Command::cargo_bin("listent").unwrap();
    let output = cmd.args(&["--monitor", "--json", "--interval", "1.0"])
        .timeout(Duration::from_secs(3))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    // Parse output to validate JSON format
    let output_str = String::from_utf8(output).unwrap();
    
    // Each non-empty line should be valid JSON
    for line in output_str.lines() {
        if line.trim().is_empty() {
            continue;
        }
        
        // Should be parseable as JSON
        let _: serde_json::Value = serde_json::from_str(line)
            .expect(&format!("Invalid JSON line: {}", line));
    }
}

#[test]
fn test_monitor_quiet_mode_suppresses_startup() {
    let mut cmd = Command::cargo_bin("listent").unwrap();
    cmd.args(&["--monitor", "--quiet", "--interval", "1.0"])
        .timeout(Duration::from_secs(2))
        .assert()
        .success()
        .stdout(predicate::str::contains("Starting process monitoring").not())
        .stdout(predicate::str::contains("Press Ctrl+C").not());
}

#[test]
fn test_monitor_human_readable_format() {
    let mut cmd = Command::cargo_bin("listent").unwrap();
    cmd.args(&["--monitor", "--interval", "1.0"])
        .timeout(Duration::from_secs(2))
        .assert()
        .success()
        .stdout(predicate::str::contains("Starting process monitoring"))
        .stdout(predicate::str::contains("Press Ctrl+C"));
}

#[test]
fn test_monitor_error_message_formatting() {
    // Test error messages are properly formatted
    let mut cmd = Command::cargo_bin("listent").unwrap();
    cmd.args(&["--monitor", "--interval", "0.05"]) // Invalid interval
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid interval"))
        .stderr(predicate::str::contains("0.1 and 300.0"));
}

#[test]
fn test_monitor_real_time_output_streaming() {
    // Test that output appears in real-time, not buffered
    let mut cmd = Command::cargo_bin("listent").unwrap();
    cmd.args(&["--monitor", "--interval", "0.5"])
        .timeout(Duration::from_secs(2))
        .assert()
        .success(); // Basic test that streaming works
}

#[test]
fn test_monitor_json_with_quiet_mode() {
    let mut cmd = Command::cargo_bin("listent").unwrap();
    cmd.args(&["--monitor", "--json", "--quiet", "--interval", "1.0"])
        .timeout(Duration::from_secs(2))
        .assert()
        .success(); // Should still output JSON events in quiet mode
}

#[test]
fn test_monitor_output_with_filters() {
    // Test output format when filters are applied
    let mut cmd = Command::cargo_bin("listent").unwrap();
    cmd.args(&[
        "--monitor",
        "-p", "/Applications",
        "-e", "camera",
        "--interval", "1.0"
    ])
    .timeout(Duration::from_secs(2))
    .assert()
    .success()
    .stdout(predicate::str::contains("Monitoring /Applications"))
    .stdout(predicate::str::contains("camera"));
}