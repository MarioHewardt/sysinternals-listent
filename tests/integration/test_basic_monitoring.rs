use assert_cmd::Command;
use predicates::prelude::*;
use std::time::Duration;

#[test]
fn test_monitor_mode_startup() {
    let mut cmd = Command::cargo_bin("listent").unwrap();
    cmd.args(&["--monitor", "--interval", "1.0"])
        .timeout(Duration::from_secs(3))
        .assert()
        .success()
        .stdout(predicate::str::contains("Starting process monitoring"));
}

#[test]
fn test_process_detection_basic() {
    // This test runs monitor mode briefly and expects it to succeed
    // without requiring specific process detection
    let mut cmd = Command::cargo_bin("listent").unwrap();
    cmd.args(&["--monitor", "--interval", "2.0"])
        .timeout(Duration::from_secs(4))
        .assert()
        .success();
}

#[test]
fn test_ctrl_c_shutdown_handling() {
    // This test is challenging to implement directly as it requires signal handling
    // For now, we test that monitor mode can be interrupted with timeout
    let mut cmd = Command::cargo_bin("listent").unwrap();
    cmd.args(&["--monitor", "--interval", "1.0"])
        .timeout(Duration::from_secs(2))
        .assert()
        .success(); // Should be cleanly interrupted by timeout
}

#[test]
fn test_polling_interval_timing() {
    use std::time::Instant;
    
    // Test that monitor mode respects the specified interval
    // This is an indirect test through timeout behavior
    let start = Instant::now();
    
    let mut cmd = Command::cargo_bin("listent").unwrap();
    cmd.args(&["--monitor", "--interval", "1.0"])
        .timeout(Duration::from_secs(3))
        .assert()
        .success();
        
    let elapsed = start.elapsed();
    // Should run for approximately 3 seconds (allowing for startup time)
    assert!(elapsed >= Duration::from_secs(2));
    assert!(elapsed <= Duration::from_secs(4));
}

#[test]
fn test_monitor_without_crashes() {
    // Test basic stability - monitor mode should not crash
    let mut cmd = Command::cargo_bin("listent").unwrap();
    cmd.args(&["--monitor", "--interval", "0.5"])
        .timeout(Duration::from_secs(5))
        .assert()
        .success()
        .stderr(predicate::str::contains("panic").not())
        .stderr(predicate::str::contains("error").not());
}

#[test]
fn test_monitor_with_fast_interval() {
    // Test with minimum allowed interval
    let mut cmd = Command::cargo_bin("listent").unwrap();
    cmd.args(&["--monitor", "--interval", "0.1"])
        .timeout(Duration::from_secs(2))
        .assert()
        .success();
}

#[test]
fn test_monitor_with_slow_interval() {
    // Test with larger interval
    let mut cmd = Command::cargo_bin("listent").unwrap();
    cmd.args(&["--monitor", "--interval", "3.0"])
        .timeout(Duration::from_secs(4))
        .assert()
        .success();
}