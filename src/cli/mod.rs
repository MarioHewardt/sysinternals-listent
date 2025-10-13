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
use crate::models::{ScanFilters, PollingConfiguration, ScanConfig};
use crate::constants::{MIN_POLLING_INTERVAL, MAX_POLLING_INTERVAL, DEFAULT_SCAN_PATHS, format_validation_error};
use std::time::Duration;

/// Command line arguments for listent
#[derive(Parser)]
#[command(author, version, about)]
#[command(long_about = "A fast Sysinternals command-line tool to discover and list code signing entitlements for macOS executable binaries.

OPERATING MODES:
  1. Static Scan Mode (default)    - Scan files/directories for entitlements
     Usage: listent [PATH...] [--entitlement KEY] [--json] [--quiet]
     
  2. Real-time Monitor Mode        - Monitor new processes for entitlements  
     Usage: listent --monitor [--interval SECONDS] [PATH...] [--entitlement KEY]
     
  3. Background Daemon Mode        - Run monitoring as persistent daemon
     Usage: listent --daemon [--interval SECONDS] [PATH...] [--entitlement KEY]

ENTITLEMENT FILTERING EXAMPLES:
  listent /usr/bin -e \"com.apple.security.network.client\"     # Exact match
  listent /Applications -e \"com.apple.security.*\"              # All security entitlements
  listent /usr/bin /Applications -e \"*network*\"                # Multiple paths + patterns
  listent -e \"com.apple.private.*\" -e \"*.debug.*\"            # Multiple patterns (OR logic)

NOTE: Always quote patterns containing wildcards (*?[]) to prevent shell expansion.")]
pub struct Args {
    /// Directory or file paths to scan (default: /Applications)
    /// 
    /// Supports multiple paths: listent /path1 /path2 /path3
    #[arg(value_name = "PATH")]
    pub path: Vec<PathBuf>,

    /// Filter by entitlement key (exact match or glob pattern)
    /// 
    /// Supports exact matching (e.g., "com.apple.security.network.client") and
    /// glob patterns (e.g., "com.apple.security.*", "*network*", "*.client").
    /// 
    /// Multiple filters: -e key1 -e key2 OR -e key1,key2 (logical OR)
    #[arg(short, long, value_name = "KEY", value_delimiter = ',')]
    pub entitlement: Vec<String>,

    /// Output in JSON format
    #[arg(short, long)]
    pub json: bool,

    /// Suppress warnings about unreadable files
    #[arg(short, long)]
    pub quiet: bool,

    /// Enable real-time process monitoring mode
    #[arg(short, long)]
    pub monitor: bool,

    /// Polling interval in seconds (0.1 - 300.0) [monitoring mode only]
    #[arg(long, default_value = "1.0", value_name = "SECONDS")]
    pub interval: f64,

    /// Run as background daemon (implies --monitor)
    #[arg(long)]
    pub daemon: bool,

    /// Install as LaunchD service (requires --daemon and sudo)
    #[arg(long)]
    pub launchd: bool,
}

impl Args {
}

/// Parse command line arguments for static scan mode
/// Parse command line arguments for static scan mode
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

    // Validate entitlement filters if provided
    if !args.entitlement.is_empty() {
        if let Err(e) = crate::entitlements::pattern_matcher::validate_entitlement_filters(&args.entitlement) {
            return Err(anyhow::anyhow!(format_validation_error("entitlement filter", 
                &args.entitlement.join(", "), &e.to_string())));
        }
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
    
    // Validate that monitor mode is enabled and daemon mode is not
    if !args.monitor || args.daemon {
        return Err(anyhow!("--monitor flag is required for monitor mode (and --daemon must not be used)"));
    }

    // Validate interval range
    if args.interval < MIN_POLLING_INTERVAL || args.interval > MAX_POLLING_INTERVAL {
        return Err(crate::models::invalid_interval_error(args.interval));
    }

    // Validate entitlement filters if provided
    if !args.entitlement.is_empty() {
        if let Err(e) = crate::entitlements::pattern_matcher::validate_entitlement_filters(&args.entitlement) {
            return Err(anyhow::anyhow!(format_validation_error("entitlement filter", 
                &args.entitlement.join(", "), &e.to_string())));
        }
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

/// Parse command line arguments and return daemon configuration
pub fn parse_daemon_config() -> Result<(f64, Vec<PathBuf>, Vec<String>, bool)> {
    let args = Args::parse();
    
    // Validate that daemon mode is enabled
    if !args.daemon {
        return Err(anyhow!("--daemon flag is required for daemon mode"));
    }

    // Use existing path and entitlement arguments (same as monitor/scan modes)
    let paths = if args.path.is_empty() {
        // Default to /Applications if no paths specified (consistent with scan mode)
        vec![PathBuf::from("/Applications")]
    } else {
        args.path
    };
    
    // Use existing entitlement arguments
    let entitlements = args.entitlement;
    
    // Validate interval range (use existing --interval argument)
    if args.interval < MIN_POLLING_INTERVAL || args.interval > MAX_POLLING_INTERVAL {
        return Err(anyhow!("--interval must be between {} and {} seconds", MIN_POLLING_INTERVAL, MAX_POLLING_INTERVAL));
    }

    // Validate paths exist
    for path in &paths {
        if !path.exists() {
            return Err(anyhow!("Path does not exist: {}", path.display()));
        }
    }

    // Validate entitlement filters if provided
    if !entitlements.is_empty() {
        if let Err(e) = crate::entitlements::pattern_matcher::validate_entitlement_filters(&entitlements) {
            return Err(anyhow::anyhow!(format_validation_error("entitlement filter", 
                &entitlements.join(", "), &e.to_string())));
        }
    }

    Ok((args.interval, paths, entitlements, args.launchd))
}

/// Validate CLI arguments for compatibility
fn validate_args_compatibility(args: &Args) -> Result<()> {
    // Monitor mode specific validation (applies to both monitor and daemon modes)
    if (args.monitor || args.daemon) && (args.interval < MIN_POLLING_INTERVAL || args.interval > MAX_POLLING_INTERVAL) {
        return Err(crate::models::invalid_interval_error(args.interval));
    }

    // Cannot use both daemon and monitor flags together
    if args.daemon && args.monitor {
        return Err(anyhow!("Cannot use both --daemon and --monitor flags (daemon implies monitoring)"));
    }

    // LaunchD flag requires daemon mode
    if args.launchd && !args.daemon {
        return Err(anyhow!("--launchd flag requires --daemon mode"));
    }

    Ok(())
}

/// Get execution mode based on CLI arguments
pub fn get_execution_mode() -> Result<ExecutionMode> {
    let args = Args::parse();
    
    // Validate argument compatibility
    validate_args_compatibility(&args)?;
    
    if args.daemon {
        Ok(ExecutionMode::Daemon)
    } else if args.monitor {
        Ok(ExecutionMode::Monitor)
    } else {
        Ok(ExecutionMode::Scan)
    }
}

/// Execution modes for the application
#[derive(Debug)]
pub enum ExecutionMode {
    Scan,
    Monitor,
    Daemon,
}