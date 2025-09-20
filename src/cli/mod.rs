//! CLI argument parsing and validation module
//! 
//! Handles command-line interface using clap, including:
//! - Path filtering options
//! - Entitlement filtering options  
//! - Output format selection (human/JSON)
//! - Verbosity and quiet modes
//! - Monitor mode with real-time process detection
//! - Help and version commands

use clap::Parser;
use std::path::PathBuf;
use anyhow::{Result, anyhow};
use crate::models::{ScanConfig, ScanFilters, PollingConfiguration, MonitorError};
use std::time::Duration;

/// Default directories to scan if no paths are provided
const DEFAULT_SCAN_PATHS: &[&str] = &[
    "/Applications",
    "/System/Applications", 
    "/System/Library/CoreServices",
    "/usr/bin",
    "/usr/local/bin",
];

/// Command line arguments for listent
#[derive(Parser)]
#[command(author, version, about)]
#[command(long_about = "A fast command-line tool to discover and list code signing entitlements for macOS executable binaries.")]
pub struct Args {
    /// Directory or file path to scan
    #[arg(short, long, value_name = "PATH")]
    pub path: Vec<PathBuf>,

    /// Filter by entitlement key (exact match)
    #[arg(short, long, value_name = "KEY")]
    pub entitlement: Vec<String>,

    /// Output in JSON format
    #[arg(short, long)]
    pub json: bool,

    /// Suppress warnings about unreadable files
    #[arg(short, long)]
    pub quiet: bool,

    /// Enable real-time process monitoring mode
    #[arg(long)]
    pub monitor: bool,

    /// Polling interval in seconds (0.1 - 300.0) [default: 1.0]
    #[arg(long, default_value = "1.0", value_name = "SECONDS")]
    pub interval: f64,
}

/// Parse command line arguments and return scan configuration
pub fn parse_args() -> Result<ScanConfig> {
    let args = Args::parse();
    
    // Validate that --interval requires --monitor
    if args.interval != 1.0 && !args.monitor {
        return Err(anyhow!("--interval requires --monitor"));
    }

    // If monitor mode is enabled, this function shouldn't be called
    if args.monitor {
        return Err(anyhow!("Internal error: parse_args() called in monitor mode"));
    }

    // Validate paths if provided
    let mut scan_paths = Vec::new();
    if !args.path.is_empty() {
        for path in &args.path {
            if !path.exists() {
                return Err(anyhow!("Path does not exist: {}", path.display()));
            }
            scan_paths.push(path.display().to_string());
        }
    } else {
        // Use default paths
        scan_paths.extend(DEFAULT_SCAN_PATHS.iter().map(|s| s.to_string()));
    }

    let filters = ScanFilters {
        entitlements: args.entitlement,
    };

    Ok(ScanConfig {
        scan_paths,
        filters,
        json_output: args.json,
        quiet_mode: args.quiet,
    })
}

/// Parse command line arguments and return monitor configuration
pub fn parse_monitor_config() -> Result<PollingConfiguration> {
    let args = Args::parse();
    
    // Validate that monitor mode is enabled
    if !args.monitor {
        return Err(anyhow!("--monitor flag is required for monitor mode"));
    }

    // Validate interval range
    if args.interval < 0.1 || args.interval > 300.0 {
        return Err(MonitorError::InvalidInterval(args.interval).into());
    }

    // Validate paths if provided
    let mut path_filters = Vec::new();
    if !args.path.is_empty() {
        for path in &args.path {
            if !path.exists() {
                return Err(anyhow!("Path does not exist: {}", path.display()));
            }
            path_filters.push(path.clone());
        }
    }

    Ok(PollingConfiguration {
        interval: Duration::from_secs_f64(args.interval),
        path_filters,
        entitlement_filters: args.entitlement,
        output_json: args.json,
        quiet_mode: args.quiet,
    })
}

/// Check if monitor mode is enabled from CLI args
pub fn is_monitor_mode() -> bool {
    let args = Args::parse();
    args.monitor
}