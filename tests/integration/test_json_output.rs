use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;
use serde_json::Value;

#[test]
fn test_json_output_flag() {
    let temp = TempDir::new().unwrap();
    
    let mut cmd = Command::cargo_bin("listent").unwrap();
    cmd.arg("--json")
       .arg(temp.path().to_str().unwrap());
    
    cmd.assert()
        .success()
        .stdout(predicate::str::is_match(r#"\{"results":\[.*\],"summary":\{.*\}\}"#).unwrap());
}

#[test]
fn test_json_output_is_valid_json() {
    let temp = TempDir::new().unwrap();
    
    let mut cmd = Command::cargo_bin("listent").unwrap();
    cmd.arg("--json")
       .arg(temp.path().to_str().unwrap());
    
    let output = cmd.assert().success().get_output().stdout.clone();
    let json_str = String::from_utf8(output).unwrap();
    
    // Should parse as valid JSON
    let _: Value = serde_json::from_str(&json_str)
        .expect("JSON output should be valid JSON");
}

#[test]
fn test_json_output_deterministic_ordering() {
    let temp = TempDir::new().unwrap();
    
    // Run the same command twice
    let mut cmd1 = Command::cargo_bin("listent").unwrap();
    cmd1.arg("--json").arg(temp.path().to_str().unwrap());
    let output1 = cmd1.assert().success().get_output().stdout.clone();
    
    let mut cmd2 = Command::cargo_bin("listent").unwrap();
    cmd2.arg("--json").arg(temp.path().to_str().unwrap());
    let output2 = cmd2.assert().success().get_output().stdout.clone();
    
    // Results should be identical (deterministic ordering)
    assert_eq!(output1, output2, "JSON output should be deterministic");
}

#[test]
fn test_json_output_with_filters() {
    let temp = TempDir::new().unwrap();
    
    let mut cmd = Command::cargo_bin("listent").unwrap();
    cmd.arg("--json")
       .arg(temp.path().to_str().unwrap())
       .arg("--entitlement").arg("com.apple.security.app-sandbox");
    
    let output = cmd.assert().success().get_output().stdout.clone();
    let json_str = String::from_utf8(output).unwrap();
    let json: Value = serde_json::from_str(&json_str).unwrap();
    
    // Should still have valid structure with filters
    assert!(json.get("results").unwrap().is_array());
    assert!(json.get("summary").unwrap().is_object());
}

#[test]
fn test_json_vs_human_output_different() {
    let temp = TempDir::new().unwrap();
    
    // Human output
    let mut cmd1 = Command::cargo_bin("listent").unwrap();
    cmd1.arg(temp.path().to_str().unwrap());
    let human_output = cmd1.assert().success().get_output().stdout.clone();
    
    // JSON output  
    let mut cmd2 = Command::cargo_bin("listent").unwrap();
    cmd2.arg("--json").arg(temp.path().to_str().unwrap());
    let json_output = cmd2.assert().success().get_output().stdout.clone();
    
    // Should be different formats
    assert_ne!(human_output, json_output, "JSON and human output should differ");
    
    // Human should contain "---" separator, JSON should not
    let human_str = String::from_utf8(human_output).unwrap();
    let json_str = String::from_utf8(json_output).unwrap();
    
    assert!(human_str.contains("---"), "Human output should have summary separator");
    assert!(!json_str.contains("---"), "JSON output should not have summary separator");
}