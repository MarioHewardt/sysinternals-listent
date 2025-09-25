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

/// Check if a file is a supported binary format
fn check_file(path: &Path) -> Option<DiscoveredBinary> {
    if !is_executable_binary(path) {
        return None;
    }
    
    Some(DiscoveredBinary {
        path: path.to_path_buf(),
    })
}

/// Check if a file is an executable binary on macOS
fn is_executable_binary(path: &Path) -> bool {
    let metadata = match std::fs::metadata(path) {
        Ok(m) => m,
        Err(_) => return false,
    };
    
    // Check if it's a file (not directory or symlink)
    if !metadata.is_file() {
        return false;
    }
    
    // Check if it has execute permissions
    use std::os::unix::fs::PermissionsExt;
    let mode = metadata.permissions().mode();
    if mode & 0o111 == 0 {
        return false;
    }
    
    // Read first 4 bytes to check magic number
    let mut file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return false,
    };
    
    let mut buffer = [0u8; 4];
    if std::io::Read::read_exact(&mut file, &mut buffer).is_err() {
        return false;
    }
    
    // Check for Mach-O magic numbers
    is_macho_binary(&buffer)
}

/// Check if the file starts with a Mach-O magic number
fn is_macho_binary(magic_bytes: &[u8; 4]) -> bool {
    matches!(magic_bytes,
        [0xfe, 0xed, 0xfa, 0xce] | // MH_MAGIC (32-bit big endian)
        [0xce, 0xfa, 0xed, 0xfe] | // MH_MAGIC (32-bit little endian)
        [0xfe, 0xed, 0xfa, 0xcf] | // MH_MAGIC_64 (64-bit big endian)
        [0xcf, 0xfa, 0xed, 0xfe] | // MH_MAGIC_64 (64-bit little endian)
        [0xca, 0xfe, 0xba, 0xbe] | // FAT_MAGIC (universal binary)
        [0xbe, 0xba, 0xfe, 0xca]   // FAT_MAGIC (universal binary, swapped)
    )
}