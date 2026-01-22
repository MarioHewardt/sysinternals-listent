//! CLI argument parsing and validation module
//! 
//! Handles command-line interface using clap, including:
//! - Path filtering options
//! - Entitlement filtering options  
//! - Output format selection (human/JSON)
//! - Verbosity and quiet modes
//! - Monitor mode with real-time process detection
//! - Help and version commands

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use anyhow::{Result, anyhow, Context};
use crate::models::{ScanConfig, ScanFilters, PollingConfiguration, MonitorError};
use std::time::Duration;

/// Default directories to scan if no paths are provided
const DEFAULT_SCAN_PATHS: &[&str] = &[
    "/Applications",
];

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
     Usage: listent --daemon --monitor [--config FILE]
     Management: listent {install-daemon|daemon-status|daemon-stop}

ENTITLEMENT FILTERING EXAMPLES:
  listent /usr/bin -e \"com.apple.security.network.client\"     # Exact match
  listent /Applications -e \"com.apple.security.*\"              # All security entitlements
  listent /usr/bin /Applications -e \"*network*\"                # Multiple paths + patterns
  listent -e \"com.apple.private.*\" -e \"*.debug.*\"            # Multiple patterns (OR logic)

NOTE: Always quote patterns containing wildcards (*?[]) to prevent shell expansion.")]
pub struct Args {
    /// Daemon management subcommands
    #[command(subcommand)]
    pub command: Option<Commands>,

    // === SCAN TARGET OPTIONS ===
    /// Directory or file paths to scan (default: /Applications)
    /// 
    /// Supports multiple paths: listent /path1 /path2 /path3
    #[arg(value_name = "PATH", help_heading = "Scan Target")]
    pub path: Vec<PathBuf>,

    /// Filter by entitlement key (exact match or glob pattern)
    /// 
    /// Supports exact matching (e.g., "com.apple.security.network.client") and
    /// glob patterns (e.g., "com.apple.security.*", "*network*", "*.client").
    /// 
    /// Multiple filters: -e key1 -e key2 OR -e key1,key2 (logical OR)
    #[arg(short, long, value_name = "KEY", help_heading = "Filtering", value_delimiter = ',')]
    pub entitlement: Vec<String>,

    // === OUTPUT OPTIONS ===
    /// Output in JSON format
    #[arg(short, long, help_heading = "Output")]
    pub json: bool,

    /// Suppress warnings about unreadable files
    #[arg(short, long, help_heading = "Output")]
    pub quiet: bool,

    // === MONITORING MODE ===
    /// Enable real-time process monitoring mode
    #[arg(short, long, help_heading = "Monitoring")]
    pub monitor: bool,

    /// Polling interval in seconds (0.1 - 300.0) [monitoring mode only]
    #[arg(long, default_value = "1.0", value_name = "SECONDS", help_heading = "Monitoring")]
    pub interval: f64,

    // === DAEMON MODE ===
    /// Run as background daemon (requires --monitor)
    #[arg(long, help_heading = "Daemon")]
    pub daemon: bool,

    /// Path to daemon configuration file
    #[arg(short, long, value_name = "FILE", help_heading = "Daemon")]
    pub config: Option<PathBuf>,
}

/// Daemon management subcommands
#[derive(Subcommand, Debug, Clone)]
#[command(about = "Daemon service management commands")]
pub enum Commands {
    /// Install daemon service with LaunchD
    InstallDaemon {
        /// Path to configuration file
        #[arg(long, value_name = "FILE")]
        config: Option<PathBuf>,
    },
    /// Uninstall daemon service from LaunchD
    UninstallDaemon,
    /// Check daemon service status
    DaemonStatus,
    /// Stop running daemon process
    DaemonStop,
    /// View daemon logs
    Logs {
        /// Follow log output continuously
        #[arg(short, long)]
        follow: bool,
        /// Show logs since specific time (e.g., "1h", "30m", "2023-01-01 10:00")
        #[arg(long, value_name = "TIME")]
        since: Option<String>,
        /// Output format (json, human)
        #[arg(long, default_value = "human")]
        format: String,
    },
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

    // Validate entitlement filters if provided
    if !args.entitlement.is_empty() {
        crate::entitlements::pattern_matcher::validate_entitlement_filters(&args.entitlement)
            .context("Invalid entitlement filter")?;
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

    // Validate entitlement filters if provided
    if !args.entitlement.is_empty() {
        crate::entitlements::pattern_matcher::validate_entitlement_filters(&args.entitlement)
            .context("Invalid entitlement filter")?;
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

/// Parse raw command line arguments without processing
pub fn parse_args_raw() -> Result<Args> {
    Ok(Args::parse())
}

/// Validate CLI arguments for compatibility
fn validate_args_compatibility(args: &Args) -> Result<()> {
    // Daemon mode requires monitor mode
    if args.daemon && !args.monitor {
        return Err(anyhow!("--daemon requires --monitor flag"));
    }

    // Interval validation 
    if args.interval != 1.0 && !args.monitor {
        return Err(anyhow!("--interval requires --monitor flag"));
    }

    if args.interval < 0.1 || args.interval > 300.0 {
        return Err(MonitorError::InvalidInterval(args.interval).into());
    }

    Ok(())
}

/// Get execution mode based on CLI arguments
pub fn get_execution_mode() -> Result<ExecutionMode> {
    let args = Args::parse();
    
    // Validate argument compatibility
    validate_args_compatibility(&args)?;
    
    match args.command {
        Some(command) => Ok(ExecutionMode::Subcommand(command.clone())),
        None => {
            if args.daemon && args.monitor {
                Ok(ExecutionMode::Daemon)
            } else if args.monitor {
                Ok(ExecutionMode::Monitor)
            } else {
                Ok(ExecutionMode::Scan)
            }
        }
    }
}

/// Execution modes for the application
#[derive(Debug)]
pub enum ExecutionMode {
    Scan,
    Monitor,
    Daemon,
    Subcommand(Commands),
}

/// Validate time format for log filtering
pub fn validate_time_format(time_str: &str) -> Result<()> {
    // Simple validation for common time formats
    if time_str.ends_with('h') || time_str.ends_with('m') || time_str.ends_with('s') {
        let number_part = &time_str[..time_str.len()-1];
        number_part.parse::<u32>()
            .map_err(|_| anyhow!("Invalid time format: {}", time_str))?;
        Ok(())
    } else if time_str.contains('-') && time_str.contains(':') {
        // Basic datetime format validation (could be more robust)
        Ok(())
    } else {
        Err(anyhow!("Invalid time format: {}. Use formats like '1h', '30m', or '2023-01-01 10:00'", time_str))
    }
}