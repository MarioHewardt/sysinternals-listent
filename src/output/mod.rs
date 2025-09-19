//! Output formatting module
//!
//! Handles:
//! - Human-readable output formatting per contracts/output-human-format.md
//! - JSON output conforming to contracts/output-json-schema.json
//! - Summary statistics generation
//! - Quiet/verbose mode behavior

use anyhow::Result;
use crate::models::EntitlementScanOutput;

/// Format output in human-readable format
pub fn format_human(output: &EntitlementScanOutput) -> Result<()> {
    if output.results.is_empty() {
        println!("No binaries found with entitlements.");
    } else {
        // Group results by entitlement types for better readability
        let total_entitlements: usize = output.results.iter()
            .map(|r| r.entitlement_count)
            .sum();
        
        println!("Found {} binaries with {} total entitlements:\n", 
                output.results.len(), total_entitlements);
        
        for result in &output.results {
            println!("{}:", result.path);
            
            // Sort entitlements for consistent output
            let mut sorted_entitlements: Vec<_> = result.entitlements.iter().collect();
            sorted_entitlements.sort_by_key(|(k, _)| *k);
            
            for (key, value) in sorted_entitlements {
                match value {
                    serde_json::Value::Bool(b) => println!("  {}: {}", key, b),
                    serde_json::Value::String(s) => println!("  {}: {}", key, s),
                    serde_json::Value::Number(n) => println!("  {}: {}", key, n),
                    _ => println!("  {}: {}", key, value),
                }
            }
            println!();
        }
    }
    
    // Print summary
    let summary = &output.summary;
    println!("Scan Summary:");
    println!("  Scanned: {} files", summary.scanned);
    println!("  Matched: {} files", summary.matched);
    
    if summary.skipped_unreadable > 0 {
        println!("  Skipped (unreadable): {} files", summary.skipped_unreadable);
    }
    
    // Format duration nicely
    let duration_sec = summary.duration_ms as f64 / 1000.0;
    if duration_sec < 1.0 {
        println!("  Duration: {}ms", summary.duration_ms);
    } else {
        println!("  Duration: {:.2}s", duration_sec);
    }
    
    if let Some(true) = summary.interrupted {
        println!("  Status: Interrupted by user");
    }
    
    Ok(())
}