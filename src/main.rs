#![forbid(unsafe_code)]

mod cli;
mod models;
mod scan;
mod entitlements;
mod output;
mod monitor;
mod daemon;
mod constants;

use anyhow::{Result, Context};
use std::time::Instant;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Context for file processing operations to reduce parameter passing
struct ProcessingContext<'a> {
    config: &'a models::ScanConfig,
    results: &'a mut Vec<models::BinaryResult>,
    scanned: &'a mut usize,
    matched: &'a mut usize,
    skipped_unreadable: &'a mut usize,
    progress: &'a mut Option<output::progress::ScanProgress>,
    interrupted: &'a Arc<AtomicBool>,
}

fn main() -> Result<()> {
    // Determine execution mode from CLI arguments
    match cli::get_execution_mode()? {
        cli::ExecutionMode::Scan => run_scan_mode(),
        cli::ExecutionMode::Monitor => run_monitor_mode(),
        cli::ExecutionMode::Daemon => run_daemon_mode(),
    }
}

fn run_scan_mode() -> Result<()> {
    let config = cli::parse_args()?;
    
    // Set up interrupt handling using signal-hook
    let interrupted = Arc::new(AtomicBool::new(false));
    
    // Register signal handlers for SIGINT and SIGTERM
    signal_hook::flag::register(signal_hook::consts::SIGINT, interrupted.clone())?;
    signal_hook::flag::register(signal_hook::consts::SIGTERM, interrupted.clone())?;
    
    let start_time = Instant::now();
    
    // Progress indicator for animated scanning
    let mut progress = if !config.quiet_mode {
        Some(output::progress::ScanProgress::new())
    } else {
        None
    };
    
    let mut results = Vec::new();
    let mut scanned = 0;
    let mut matched = 0;
    let mut skipped_unreadable = 0;
    
    // Fast count total files (like find command) with interrupt support
    let total_files = scan::count_total_files_with_interrupt(&config.scan_paths, &interrupted)
        .context("Failed to count total files")?;
    
    // Check if interrupted during counting
    if interrupted.load(Ordering::Relaxed) {
        return Ok(());
    }
    
    // Start progress with total file count
    if let Some(ref mut progress) = progress {
        progress.start_scanning(total_files);
    }
    
    // Process files one by one
    for path_str in &config.scan_paths {
        let path = std::path::Path::new(path_str);
        if path.exists() {
            // Update progress to show current top-level directory (only once per directory)
            if let Some(ref mut progress) = progress {
                progress.set_current_directory(path);
            }
            
            // Create processing context
            let mut ctx = ProcessingContext {
                config: &config,
                results: &mut results,
                scanned: &mut scanned,
                matched: &mut matched,
                skipped_unreadable: &mut skipped_unreadable,
                progress: &mut progress,
                interrupted: &interrupted,
            };
            
            if path.is_file() {
                // Single file case
                process_single_file(path, &mut ctx)?;
            } else {
                // Directory case
                process_directory_files(path, &mut ctx)?;
            }
        }
        
        // Check for interruption between directories
        if interrupted.load(Ordering::Relaxed) {
            break;
        }
    }

    // Complete progress indicator
    if let Some(mut progress) = progress {
        progress.complete_scanning();
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

/// Process a single file, checking if it's a binary and extracting entitlements
fn process_single_file(path: &std::path::Path, ctx: &mut ProcessingContext) -> Result<()> {
    // Check for interruption
    if ctx.interrupted.load(Ordering::Relaxed) {
        return Ok(());
    }
    
    // Check if this file is a binary
    if let Some(binary) = scan::check_single_file(path) {
        process_binary(binary, ctx)?;
    } else {
        // Non-binary file, just increment skipped count
        if let Some(ref mut progress) = ctx.progress {
            progress.increment_skipped();
        }
    }
    
    Ok(())
}

/// Process all files in a directory recursively
fn process_directory_files(dir_path: &std::path::Path, ctx: &mut ProcessingContext) -> Result<()> {
    use std::fs;
    
    for entry in fs::read_dir(dir_path)? {
        // Check for interruption at the start of each directory entry
        if ctx.interrupted.load(Ordering::Relaxed) {
            return Ok(());
        }
        
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() {
            process_single_file(&path, ctx)?;
            
            // Check for interruption after processing each file
            if ctx.interrupted.load(Ordering::Relaxed) {
                return Ok(());
            }
        } else if path.is_dir() {
            // Recursively process subdirectories without updating progress directory name
            process_directory_files(&path, ctx)?;
            
            // Check for interruption after processing each subdirectory
            if ctx.interrupted.load(Ordering::Relaxed) {
                return Ok(());
            }
        }
    }
    
    Ok(())
}

/// Process a binary file and extract entitlements
fn process_binary(binary: scan::DiscoveredBinary, ctx: &mut ProcessingContext) -> Result<()> {
    *ctx.scanned += 1;
    
    // Update progress
    if let Some(ref mut progress) = ctx.progress {
        progress.increment_scanned();
    }
    
    // Extract entitlements
    match entitlements::extract_entitlements(&binary.path) {
        Ok(entitlement_map) => {
            // Get list of entitlement keys for pattern matching
            let entitlement_keys: Vec<String> = entitlement_map.keys().cloned().collect();
            
            // Check if any entitlements match the filters using consistent pattern matching
            if entitlements::pattern_matcher::entitlements_match_filters(&entitlement_keys, &ctx.config.filters.entitlements) {
                // Apply entitlement filters to output (only show matching entitlements)
                let filtered_entitlements = if ctx.config.filters.entitlements.is_empty() {
                    entitlement_map
                } else {
                    entitlement_map.into_iter()
                        .filter(|(key, _)| {
                            ctx.config.filters.entitlements.iter().any(|filter| {
                                entitlements::pattern_matcher::matches_entitlement_filter(key, filter)
                            })
                        })
                        .collect()
                };
                
                *ctx.matched += 1;
                ctx.results.push(models::BinaryResult {
                    path: binary.path.to_string_lossy().to_string(),
                    entitlement_count: filtered_entitlements.len(),
                    entitlements: filtered_entitlements,
                });
            }
        },
        Err(err) => {
            // Count as skipped if we can't read the entitlements
            *ctx.skipped_unreadable += 1;
            if !ctx.config.quiet_mode {
                eprintln!("Warning: Could not extract entitlements from {}: {}", 
                         binary.path.display(), err);
            }
        }
    }
    
    Ok(())
}

fn run_monitor_mode() -> Result<()> {
    let config = cli::parse_monitor_config()?;
    
    // Set up interrupt handling using signal-hook (same as scan mode)
    let interrupted = Arc::new(AtomicBool::new(false));
    
    // Register signal handlers for SIGINT and SIGTERM
    signal_hook::flag::register(signal_hook::consts::SIGINT, interrupted.clone())?;
    signal_hook::flag::register(signal_hook::consts::SIGTERM, interrupted.clone())?;
    
    monitor::polling::start_monitoring_with_interrupt(config, interrupted)
}

fn run_daemon_mode() -> Result<()> {
    // Parse daemon-specific configuration from CLI
    let (interval, paths, entitlements, launchd) = cli::parse_daemon_config()?;
    
    // Check if we're the child daemon process (suppress output for child)
    let is_child_process = std::env::var("LISTENT_DAEMON_CHILD").is_ok();
    
    if launchd {
        if !is_child_process {
            println!("🔧 Installing listent as LaunchD service...");
            println!("   Interval: {}s", interval);
            println!("   Paths: {:?}", paths);
            println!("   Entitlements: {:?}", entitlements);
        }
        
        // Create tokio runtime for async daemon operations
        let runtime = tokio::runtime::Runtime::new()
            .context("Failed to create tokio runtime")?;
            
        // Install as LaunchD service
        runtime.block_on(daemon::install_launchd_service(interval, paths, entitlements))
    } else {
        if !is_child_process {
            println!("🔧 Starting listent daemon...");
            println!("   Interval: {}s", interval);
            println!("   Paths: {:?}", paths);
            println!("   Entitlements: {:?}", entitlements);
        }
        
        // Create tokio runtime for async daemon operations
        let runtime = tokio::runtime::Runtime::new()
            .context("Failed to create tokio runtime")?;
        
        // Execute daemon mode with parsed arguments
        runtime.block_on(daemon::run_daemon_with_args(interval, paths, entitlements))
    }
}
