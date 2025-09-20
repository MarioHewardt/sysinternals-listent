use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;

#[test]
fn test_human_readable_output_format() {
    let mut cmd = Command::cargo_bin("listent").unwrap();
    cmd.args(&["--monitor", "--interval", "1.0"])
        .timeout(std::time::Duration::from_secs(2))
        .assert()
        .success()
        .stdout(predicate::str::contains("Starting process monitoring"))
        .stdout(predicate::str::contains("Press Ctrl+C to stop")); // Will fail until human output is implemented
}

#[test]
fn test_json_output_format() {
    let mut cmd = Command::cargo_bin("listent").unwrap();
    let output = cmd.args(&["--monitor", "--json", "--interval", "1.0"])
        .timeout(std::time::Duration::from_secs(2))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    // Parse each line as JSON to validate format
    let output_str = String::from_utf8(output).unwrap();
    for line in output_str.lines() {
        if line.trim().is_empty() {
            continue;
        }
        
        // Each line should be valid JSON with required fields
        let json: Value = serde_json::from_str(line).expect("Invalid JSON output");
        
        assert!(json.get("timestamp").is_some(), "Missing timestamp field");
        assert!(json.get("event_type").is_some(), "Missing event_type field");
        assert_eq!(json["event_type"], "process_detected");
        
        let process = json.get("process").expect("Missing process field");
        assert!(process.get("pid").is_some(), "Missing process.pid field");
        assert!(process.get("name").is_some(), "Missing process.name field");
        assert!(process.get("path").is_some(), "Missing process.path field");
        assert!(process.get("entitlements").is_some(), "Missing process.entitlements field");
    }
}

#[test]
fn test_timestamp_formatting() {
    let mut cmd = Command::cargo_bin("listent").unwrap();
    cmd.args(&["--monitor", "--interval", "1.0"])
        .timeout(std::time::Duration::from_secs(2))
        .assert()
        .success()
        // Timestamp should be in ISO 8601 format
        .stdout(predicate::str::is_match(r"\[\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z\]").unwrap());
}

#[test]
fn test_entitlements_list_formatting() {
    let mut cmd = Command::cargo_bin("listent").unwrap();
    cmd.args(&["--monitor", "--interval", "1.0"])
        .timeout(std::time::Duration::from_secs(2))
        .assert()
        .success()
        .stdout(predicate::str::contains("Entitlements:"));
}

#[test]
fn test_quiet_mode_output_suppression() {
    let mut cmd = Command::cargo_bin("listent").unwrap();
    cmd.args(&["--monitor", "--quiet", "--interval", "1.0"])
        .timeout(std::time::Duration::from_secs(2))
        .assert()
        .success()
        .stdout(predicate::str::contains("Starting process monitoring").not())
        .stdout(predicate::str::contains("Press Ctrl+C").not());
}

#[test]
fn test_no_entitlements_formatting() {
    // This test will need to be refined based on actual process detection
    let mut cmd = Command::cargo_bin("listent").unwrap();
    cmd.args(&["--monitor", "--interval", "1.0"])
        .timeout(std::time::Duration::from_secs(2))
        .assert()
        .success(); // Basic success test for now
}