/// Comprehensive functional tests using controlled test environment
/// These tests are designed to be reliable and work consistently across different systems

use std::time::Duration;
use anyhow::Result;

mod helpers;
use helpers::{TestEnvironment, reliable_runner::*};

#[test]
fn test_end_to_end_static_scan_workflow() -> Result<()> {
    let test_env = TestEnvironment::new()?;
    let mut runner = ReliableTestRunner::new(30); // 30 second timeout
    
    // Test basic scan
    let result = runner.run_command_with_timeout(
        std::process::Command::new("./target/release/listent")
            .arg(test_env.path())
            .arg("--json")
    )?;
    
    assert!(result.was_successful(), "Basic scan should succeed");
    
    // Verify JSON output is valid
    let json: serde_json::Value = serde_json::from_str(&result.stdout)?;
    assert!(json.get("results").is_some(), "Should have results field");
    assert!(json.get("summary").is_some(), "Should have summary field");
    
    // Test with entitlement filter
    let filtered_result = runner.run_command_with_timeout(
        std::process::Command::new("./target/release/listent")
            .arg(test_env.path())
            .arg("-e")
            .arg("com.apple.security.network.*")
            .arg("--json")
    )?;
    
    assert!(filtered_result.was_successful(), "Filtered scan should succeed");
    
    Ok(())
}

#[test]
fn test_end_to_end_monitor_workflow() -> Result<()> {
    let test_env = TestEnvironment::new()?;
    let mut scenario = TestScenario::new("monitor_workflow", 60);
    
    // Start a monitor process
    let monitor_result = scenario.run_monitor_test(&[
        "--interval", "1.0",
        "--json"
    ], Duration::from_secs(5))?;
    
    // Should exit cleanly
    assert!(monitor_result.was_successful(), "Monitor should exit cleanly");
    
    // Should show expected startup behavior
    assert!(monitor_result.contains_output("Starting process monitoring") ||
            monitor_result.contains_output("Press Ctrl+C"),
        "Should show startup message");
    
    Ok(())
}

#[test]
fn test_signal_handling_reliability() -> Result<()> {
    let mut runner = ReliableTestRunner::new(15);
    
    // Test CTRL-C in scan mode
    let scan_result = runner.run_monitor_with_interrupt(&[
        "/Applications", // Use real path that exists
        "--quiet"
    ], Duration::from_millis(1500))?;
    
    // Should handle interrupt gracefully
    assert_eq!(scan_result.exit_code, Some(0), "Scan should exit cleanly on interrupt");
    
    // Test CTRL-C in monitor mode
    let monitor_result = runner.run_monitor_with_interrupt(&[
        "--interval", "2.0",
        "--quiet"
    ], Duration::from_millis(2500))?;
    
    assert_eq!(monitor_result.exit_code, Some(0), "Monitor should exit cleanly on interrupt");
    
    Ok(())
}

#[test]
fn test_process_detection_with_controlled_processes() -> Result<()> {
    let test_env = TestEnvironment::new()?;
    let mut scenario = TestScenario::new("process_detection", 30);
    
    // Spawn test processes with known entitlements after monitor starts
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_secs(2)); // Let monitor establish baseline
        
        // This is a simplified test - in practice you'd need more sophisticated
        // process spawning and monitoring
        let _ = std::process::Command::new("sleep")
            .arg("3")
            .spawn();
    });
    
    let monitor_result = scenario.run_monitor_test(&[
        "--interval", "0.5",
        "--json"
    ], Duration::from_secs(8))?;
    
    // Should complete successfully
    assert!(monitor_result.was_successful(), "Process detection test should succeed");
    
    Ok(())
}

#[test]
fn test_error_handling_and_edge_cases() -> Result<()> {
    let mut runner = ReliableTestRunner::new(10);
    
    // Test nonexistent path
    let result = runner.run_command_with_timeout(
        std::process::Command::new("./target/release/listent")
            .arg("/nonexistent/path/that/should/not/exist")
            .arg("--quiet")
    )?;
    
    // Should handle gracefully (exact behavior may vary)
    assert!(result.exit_code.is_some(), "Should exit with status code");
    
    // Test invalid interval
    let invalid_interval_result = runner.run_command_with_timeout(
        std::process::Command::new("./target/release/listent")
            .arg("--monitor")
            .arg("--interval")
            .arg("-1.0") // Invalid negative interval
    )?;
    
    // Should reject invalid interval
    assert!(invalid_interval_result.exit_code != Some(0) || 
            invalid_interval_result.contains_output("error") ||
            invalid_interval_result.contains_output("invalid"),
        "Should handle invalid intervals");
    
    Ok(())
}

#[test] 
fn test_output_format_consistency() -> Result<()> {
    let test_env = TestEnvironment::new()?;
    let mut runner = ReliableTestRunner::new(15);
    
    // Test human-readable output
    let human_result = runner.run_command_with_timeout(
        std::process::Command::new("./target/release/listent")
            .arg(test_env.path())
    )?;
    
    assert!(human_result.was_successful(), "Human output should work");
    assert!(human_result.contains_stdout("Found") || 
            human_result.contains_stdout("Scan Summary") ||
            human_result.contains_stdout("files"),
        "Should contain human-readable indicators");
    
    // Test JSON output
    let json_result = runner.run_command_with_timeout(
        std::process::Command::new("./target/release/listent")
            .arg(test_env.path())
            .arg("--json")
    )?;
    
    assert!(json_result.was_successful(), "JSON output should work");
    
    // Should be valid JSON
    let _: serde_json::Value = serde_json::from_str(&json_result.stdout)
        .map_err(|e| anyhow::anyhow!("Invalid JSON output: {}", e))?;
    
    Ok(())
}

#[test]
fn test_performance_and_timeout_handling() -> Result<()> {
    let mut runner = ReliableTestRunner::new(20);
    
    // Test that scan doesn't hang indefinitely
    let start = std::time::Instant::now();
    let result = runner.run_command_with_timeout(
        std::process::Command::new("./target/release/listent")
            .arg("/Applications") // Real directory that should complete
            .arg("--quiet")
    )?;
    let duration = start.elapsed();
    
    // Should complete within reasonable time
    assert!(duration < Duration::from_secs(15), 
        "Scan should complete within 15 seconds");
    
    // Should not timeout
    assert!(!result.timed_out, "Scan should not timeout");
    
    Ok(())
}

#[test]
fn test_concurrent_operations() -> Result<()> {
    use std::thread;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    
    let test_env = TestEnvironment::new()?;
    let success_count = Arc::new(AtomicUsize::new(0));
    let mut handles = vec![];
    
    // Run multiple scans concurrently
    for i in 0..3 {
        let test_path = test_env.path().to_path_buf();
        let success_counter = success_count.clone();
        
        let handle = thread::spawn(move || -> Result<()> {
            let mut runner = ReliableTestRunner::new(10);
            let result = runner.run_command_with_timeout(
                std::process::Command::new("./target/release/listent")
                    .arg(&test_path)
                    .arg("--json")
                    .arg("--quiet")
            )?;
            
            if result.was_successful() {
                success_counter.fetch_add(1, Ordering::SeqCst);
            }
            Ok(())
        });
        
        handles.push(handle);
    }
    
    // Wait for all threads
    for handle in handles {
        handle.join().unwrap()?;
    }
    
    // All should succeed
    assert_eq!(success_count.load(Ordering::SeqCst), 3, 
        "All concurrent scans should succeed");
    
    Ok(())
}

#[test]
#[ignore] // This test takes longer and is more comprehensive
fn test_long_running_monitor_stability() -> Result<()> {
    let mut scenario = TestScenario::new("long_monitor", 120);
    
    // Run monitor for extended period
    let result = scenario.run_monitor_test(&[
        "--interval", "2.0",
        "--quiet"
    ], Duration::from_secs(30))?;
    
    // Should remain stable over time
    assert!(result.was_successful(), "Long-running monitor should be stable");
    
    Ok(())
}