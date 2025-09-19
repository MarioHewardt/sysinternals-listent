use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_version_prints_semantic_version() {
    let mut cmd = Command::cargo_bin("listent").unwrap();
    cmd.arg("--version");
    
    cmd.assert()
        .success()
        .stdout(predicate::str::is_match(r"listent \d+\.\d+\.\d+").unwrap());
}

#[test]
#[ignore] // Git hash not implemented yet
fn test_version_includes_commit_hash() {
    let mut cmd = Command::cargo_bin("listent").unwrap();
    cmd.arg("--version");
    
    // Should include git hash in format like "listent 0.1.0 (abc1234)"
    cmd.assert()
        .success()
        .stdout(predicate::str::is_match(r"listent \d+\.\d+\.\d+ \([a-f0-9]+\)").unwrap());
}

#[test]
fn test_short_version_flag() {
    let mut cmd = Command::cargo_bin("listent").unwrap();
    cmd.arg("-V");
    
    cmd.assert()
        .success()
        .stdout(predicate::str::is_match(r"listent \d+\.\d+\.\d+").unwrap());
}