use assert_cmd::Command;
use predicates::prelude::*;
use std::env;

#[test]
fn test_default_scan_directories() {
    // Test scanning default directories when no --path specified
    let mut cmd = Command::cargo_bin("listent").unwrap();
    
    // Don't specify --path, should use defaults
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Scanned:"));
}

#[test]
fn test_default_directories_listed_in_help() {
    // Help should document what the default directories are
    let mut cmd = Command::cargo_bin("listent").unwrap();
    cmd.arg("--help");
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("/Applications")
                .or(predicate::str::contains("default"))
                .or(predicate::str::contains("application directories")));
}

#[test]
fn test_default_scan_respects_environment_override() {
    // For testing, should be able to override default directories
    // This prevents tests from scanning the entire system
    let mut cmd = Command::cargo_bin("listent").unwrap();
    cmd.env("LISTENT_DEFAULT_DIRS", "/tmp");
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Scanned:"));
}

#[test]
fn test_default_scan_produces_summary() {
    let mut cmd = Command::cargo_bin("listent").unwrap();
    
    cmd.assert()
        .success()
        .stdout(predicate::str::is_match(r"Scanned: \d+").unwrap())
        .stdout(predicate::str::is_match(r"Matched: \d+").unwrap())
        .stdout(predicate::str::is_match(r"Duration: \d+ms").unwrap());
}