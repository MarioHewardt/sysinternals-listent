//! Filesystem scanning and binary discovery module
//!
//! Responsible for:
//! - Traversing directory trees to find executable binaries
//! - Filtering by file type (Mach-O, bundles)
//! - Applying path filters
//! - Deterministic ordering of results
//! - Integration with entitlement extraction

use std::fs;
use std::path::{Path, PathBuf};
use std::os::unix::fs::PermissionsExt;
use anyhow::Result;

/// Represents a discovered binary file
#[derive(Debug, Clone)]
pub struct DiscoveredBinary {
    pub path: PathBuf,
}

/// Scan a list of directories for executable binaries
pub fn scan_directories(paths: &[String]) -> Result<Vec<DiscoveredBinary>> {
    let mut binaries = Vec::new();
    
    for path_str in paths {
        let path = Path::new(path_str);
        if path.exists() {
            if path.is_file() {
                // Single file case
                if let Some(binary) = check_file(path) {
                    binaries.push(binary);
                }
            } else if path.is_dir() {
                // Directory case - scan recursively
                scan_directory_recursive(path, &mut binaries)?;
            }
        }
    }
    
    // Sort for deterministic ordering
    binaries.sort_by(|a, b| a.path.cmp(&b.path));
    
    Ok(binaries)
}

/// Recursively scan a directory for binaries
fn scan_directory_recursive(dir: &Path, binaries: &mut Vec<DiscoveredBinary>) -> Result<()> {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return Ok(()), // Skip unreadable directories
    };
    
    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue, // Skip unreadable entries
        };
        
        let path = entry.path();
        
        if path.is_file() {
            if let Some(binary) = check_file(&path) {
                binaries.push(binary);
            }
        } else if path.is_dir() {
            // Skip some common directories that won't contain executables
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if should_skip_directory(name) {
                    continue;
                }
            }
            
            // Recurse into subdirectory
            let _ = scan_directory_recursive(&path, binaries);
        }
    }
    
    Ok(())
}

/// Check if a file is a binary we should examine
fn check_file(path: &Path) -> Option<DiscoveredBinary> {
    // Check if file is executable
    let metadata = match fs::metadata(path) {
        Ok(metadata) => metadata,
        Err(_) => return None,
    };
    
    let is_executable = metadata.permissions().mode() & 0o111 != 0;
    
    // Quick Mach-O detection by reading file header
    let is_mach_o = is_likely_mach_o(path);
    
    // Include if it's executable or appears to be a Mach-O binary
    if is_executable || is_mach_o {
        Some(DiscoveredBinary {
            path: path.to_path_buf(),
        })
    } else {
        None
    }
}

/// Quick check if file might be a Mach-O binary by reading magic bytes
fn is_likely_mach_o(path: &Path) -> bool {
    let mut file = match fs::File::open(path) {
        Ok(file) => file,
        Err(_) => return false,
    };
    
    use std::io::Read;
    let mut buffer = [0u8; 4];
    if file.read_exact(&mut buffer).is_err() {
        return false;
    }
    
    // Check for Mach-O magic numbers
    matches!(buffer, 
        [0xfe, 0xed, 0xfa, 0xce] | // MH_MAGIC (32-bit big endian)
        [0xce, 0xfa, 0xed, 0xfe] | // MH_MAGIC (32-bit little endian)
        [0xfe, 0xed, 0xfa, 0xcf] | // MH_MAGIC_64 (64-bit big endian)
        [0xcf, 0xfa, 0xed, 0xfe] | // MH_MAGIC_64 (64-bit little endian)
        [0xca, 0xfe, 0xba, 0xbe] | // FAT_MAGIC (universal binary)
        [0xbe, 0xba, 0xfe, 0xca]   // FAT_MAGIC (universal binary, swapped)
    )
}

/// Determine if we should skip scanning a directory
fn should_skip_directory(name: &str) -> bool {
    matches!(name,
        ".git" | ".svn" | ".hg" | // Version control
        "node_modules" | "target" | "build" | "dist" | // Build artifacts
        ".DS_Store" | ".Trash" | // macOS system
        "Cache" | "Caches" | "cache" | "caches" | // Cache directories
        "Logs" | "logs" | "log" | // Log directories
        "tmp" | "temp" | "temporary" | // Temporary directories
        "test" | "tests" | "spec" | "specs" | // Test directories
        ".bundle" | ".gradle" | ".maven" | // Build tools
        "Backup" | "backup" | "backups" | // Backup directories
        "Documentation" | "docs" | "man" | // Documentation
        "locale" | "locales" | "lang" | "language" // Localization
    ) || name.starts_with('.') && name.len() > 1 // Hidden directories except current/parent
}