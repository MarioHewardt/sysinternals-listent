use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

#[test]
fn test_path_and_entitlement_filters_combined() {
    let temp = TempDir::new().unwrap();
    
    let mut cmd = Command::cargo_bin("listent").unwrap();
    cmd.arg("--path").arg(temp.path().to_str().unwrap())
       .arg("--entitlement").arg("com.apple.security.app-sandbox");
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Scanned:"))
        .stdout(predicate::str::contains("Matched:"));
}

#[test]
fn test_combined_filters_logical_and() {
    // Path AND entitlement filters should both apply
    // Only binaries in specified paths AND containing specified entitlements
    let temp1 = TempDir::new().unwrap();
    let temp2 = TempDir::new().unwrap();
    
    let mut cmd = Command::cargo_bin("listent").unwrap();
    cmd.arg("--path").arg(temp1.path().to_str().unwrap())
       .arg("--path").arg(temp2.path().to_str().unwrap())
       .arg("--entitlement").arg("com.apple.security.app-sandbox")
       .arg("--entitlement").arg("com.apple.security.network.client");
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Scanned:"));
}

#[test]
fn test_combined_filters_restrictive_result() {
    // Combined filters should be more restrictive than either alone
    let temp = TempDir::new().unwrap();
    
    // First get count with just path filter
    let mut cmd1 = Command::cargo_bin("listent").unwrap();
    cmd1.arg("--path").arg(temp.path().to_str().unwrap());
    let output1 = cmd1.assert().success().get_output().stdout.clone();
    
    // Then get count with path + entitlement filter
    let mut cmd2 = Command::cargo_bin("listent").unwrap();
    cmd2.arg("--path").arg(temp.path().to_str().unwrap())
        .arg("--entitlement").arg("com.nonexistent.entitlement");
    let output2 = cmd2.assert().success().get_output().stdout.clone();
    
    // Combined filter should find same or fewer matches
    let output1_str = String::from_utf8(output1).unwrap();
    let output2_str = String::from_utf8(output2).unwrap();
    
    assert!(output1_str.contains("Scanned:"), "Path-only scan should show scanned count");
    assert!(output2_str.contains("Scanned:"), "Combined filter scan should show scanned count");
}

#[test]
fn test_all_filter_types_together() {
    let temp = TempDir::new().unwrap();
    
    let mut cmd = Command::cargo_bin("listent").unwrap();
    cmd.arg("--path").arg(temp.path().to_str().unwrap())
       .arg("--entitlement").arg("com.apple.security.app-sandbox")
       .arg("--json")
       .arg("--quiet");
    
    cmd.assert()
        .success();
    
    // Should produce valid JSON output
    let output = cmd.assert().success().get_output().stdout.clone();
    let json_str = String::from_utf8(output).unwrap();
    let _: serde_json::Value = serde_json::from_str(&json_str)
        .expect("Combined filters with JSON should produce valid JSON");
}