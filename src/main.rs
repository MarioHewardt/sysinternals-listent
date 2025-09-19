#![forbid(unsafe_code)]

mod cli;
mod models;
mod scan;
mod entitlements;
mod output;

use anyhow::Result;
use std::time::Instant;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

fn main() -> Result<()> {
    let config = cli::parse_args()?;
    
    // Set up interrupt handling
    let interrupted = Arc::new(AtomicBool::new(false));
    let interrupted_clone = interrupted.clone();
    
    let _ = signal_hook::flag::register(signal_hook::consts::SIGINT, interrupted_clone);
    let _ = signal_hook::flag::register(signal_hook::consts::SIGTERM, interrupted.clone());
    
    let start_time = Instant::now();
    
    // Scan directories for binaries
    let discovered_binaries = scan::scan_directories(&config.scan_paths)?;
    
    let mut results = Vec::new();
    let mut scanned = 0;
    let mut matched = 0;
    let mut skipped_unreadable = 0;
    
    for binary in discovered_binaries {
        // Check for interruption
        if interrupted.load(Ordering::Relaxed) {
            break;
        }
        
        scanned += 1;
        
        // Progress indicator for long operations
        if !config.quiet_mode && scanned % 100 == 0 {
            eprintln!("Processed {} files...", scanned);
        }
        
        // Extract entitlements
        match entitlements::extract_entitlements(&binary.path) {
            Ok(entitlement_map) => {
                // Apply entitlement filters if specified
                let filtered_entitlements = if config.filters.entitlements.is_empty() {
                    entitlement_map
                } else {
                    entitlement_map.into_iter()
                        .filter(|(key, _)| config.filters.entitlements.contains(key))
                        .collect()
                };
                
                // Only include binaries that have entitlements (and match filters)
                if !filtered_entitlements.is_empty() {
                    matched += 1;
                    results.push(models::BinaryResult {
                        path: binary.path.to_string_lossy().to_string(),
                        entitlement_count: filtered_entitlements.len(),
                        entitlements: filtered_entitlements,
                    });
                }
            },
            Err(err) => {
                // Count as skipped if we can't read the entitlements
                skipped_unreadable += 1;
                if !config.quiet_mode {
                    eprintln!("Warning: Could not extract entitlements from {}: {}", 
                             binary.path.display(), err);
                }
            }
        }
    }
    
    let duration_ms = start_time.elapsed().as_millis() as u64;
    let was_interrupted = interrupted.load(Ordering::Relaxed);
    
    let output = models::EntitlementScanOutput {
        results,
        summary: models::ScanSummary {
            scanned,
            matched,
            skipped_unreadable,
            duration_ms,
            interrupted: if was_interrupted { Some(true) } else { None },
        },
    };

    if config.json_output {
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        output::format_human(&output)?;
    }

    Ok(())
}
