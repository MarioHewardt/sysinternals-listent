//! CLI argument parsing and validation module
//! 
//! Handles command-line interface using clap, including:
//! - Path filtering options
//! - Entitlement filtering options  
//! - Output format selection (human/JSON)
//! - Verbosity and quiet modes
//! - Help and version commands

use clap::{Arg, ArgAction, Command};
use std::path::Path;
use anyhow::{Result, anyhow};
use crate::models::{ScanConfig, ScanFilters};

/// Default directories to scan if no paths are provided
const DEFAULT_SCAN_PATHS: &[&str] = &[
    "/Applications",
    "/System/Applications", 
    "/System/Library/CoreServices",
    "/usr/bin",
    "/usr/local/bin",
];

/// Parse command line arguments and return configuration
pub fn parse_args() -> Result<ScanConfig> {
    let matches = Command::new("listent")
        .version(env!("CARGO_PKG_VERSION"))
        .about("List entitlements for macOS binaries")
        .long_about("A fast command-line tool to discover and list code signing entitlements for macOS executable binaries.")
        .arg(
            Arg::new("path")
                .short('p')
                .long("path")
                .value_name("PATH")
                .help("Directory or file path to scan")
                .action(ArgAction::Append)
        )
        .arg(
            Arg::new("entitlement")
                .short('e')
                .long("entitlement")
                .value_name("KEY")
                .help("Filter by entitlement key (exact match)")
                .action(ArgAction::Append)
        )
        .arg(
            Arg::new("json")
                .short('j')
                .long("json")
                .help("Output in JSON format")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("quiet")
                .short('q')
                .long("quiet")
                .help("Suppress warnings about unreadable files")
                .action(ArgAction::SetTrue)
        )
        .get_matches();

    // Validate paths if provided
    let mut scan_paths = Vec::new();
    if let Some(paths) = matches.get_many::<String>("path") {
        for path_str in paths {
            let path = Path::new(path_str);
            if !path.exists() {
                return Err(anyhow!("Path does not exist: {}", path_str));
            }
            scan_paths.push(path_str.clone());
        }
    } else {
        // Use default paths
        scan_paths.extend(DEFAULT_SCAN_PATHS.iter().map(|s| s.to_string()));
    }

    // Collect entitlement filters
    let entitlements = matches.get_many::<String>("entitlement")
        .map(|values| values.cloned().collect())
        .unwrap_or_default();

    let filters = ScanFilters {
        entitlements,
    };

    Ok(ScanConfig {
        scan_paths,
        filters,
        json_output: matches.get_flag("json"),
        quiet_mode: matches.get_flag("quiet"),
    })
}