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

/// Fast file counting (like find) - only uses filesystem metadata
fn count_files_in_directory_with_interrupt(path: &Path, interrupted: &std::sync::Arc<std::sync::atomic::AtomicBool>) -> Result<usize> {
    let mut count = 0;
    
    for entry in fs::read_dir(path)? {
        // Check for interruption frequently
        if interrupted.load(std::sync::atomic::Ordering::Relaxed) {
            return Ok(count); // Return partial count on interrupt
        }
        
        let entry = entry?;
        let entry_path = entry.path();
        
        // Count files and symlinks that point to files (consistent with processing logic)
        if entry_path.is_file() {
            count += 1;
        } else if entry_path.is_dir() {
            count += count_files_in_directory_with_interrupt(&entry_path, interrupted)?;
        }
    }
    
    Ok(count)
}

/// Fast counting of total files in all scan paths with interrupt support
pub fn count_total_files_with_interrupt(scan_paths: &[String], interrupted: &std::sync::Arc<std::sync::atomic::AtomicBool>) -> Result<usize> {
    let mut total = 0;
    
    for path_str in scan_paths {
        // Check for interruption between directories
        if interrupted.load(std::sync::atomic::Ordering::Relaxed) {
            return Ok(total); // Return partial count on interrupt
        }
        
        let path = Path::new(path_str);
        if path.exists() {
            if path.is_file() {
                total += 1;
            } else if path.is_dir() {
                total += count_files_in_directory_with_interrupt(path, interrupted)?;
            }
        }
    }
    
    Ok(total)
}

/// Check a single file to see if it's a binary
pub fn check_single_file(path: &Path) -> Option<DiscoveredBinary> {
    check_file(path)
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