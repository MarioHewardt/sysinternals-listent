//! Progress indicator for static scans
//! 
//! Provides animated progress display during directory scanning operations.
//! Shows per-directory status with spinner animation and completion indicators.

use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

/// Characters for the spinning animation
const SPINNER_CHARS: &[char] = &['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];

/// Progress indicator for directory scanning
pub struct ScanProgress {
    current_directory: Option<String>,
    is_scanning: Arc<AtomicBool>,
    animation_handle: Option<thread::JoinHandle<()>>,
    quiet_mode: bool,
    total_files: usize,
    scanned_files: usize,
    skipped_files: usize,
}

impl ScanProgress {
    /// Create a new progress indicator
    pub fn new() -> Self {
        Self {
            current_directory: None,
            is_scanning: Arc::new(AtomicBool::new(false)),
            animation_handle: None,
            quiet_mode: false,
            total_files: 0,
            scanned_files: 0,
            skipped_files: 0,
        }
    }

    /// Set the total number of files to be scanned
    pub fn set_total_files(&mut self, total: usize) {
        self.total_files = total;
    }

    /// Start scanning with total file count
    pub fn start_scanning(&mut self, total_files: usize) {
        if self.quiet_mode {
            return;
        }

        self.total_files = total_files;
        self.scanned_files = 0;
        self.skipped_files = 0;
        self.is_scanning.store(true, Ordering::Relaxed);

        // Start progress display thread
        let is_scanning = Arc::clone(&self.is_scanning);
        
        self.animation_handle = Some(thread::spawn(move || {
            while is_scanning.load(Ordering::Relaxed) {
                // Just keep the thread alive for updating
                thread::sleep(Duration::from_millis(100));
            }
        }));

        // Show initial progress
        self.update_progress();
    }

    /// Update progress with current file count
    pub fn update_progress(&self) {
        if self.quiet_mode {
            return;
        }

        let processed = self.scanned_files + self.skipped_files;
        let dir_info = if let Some(ref dir) = self.current_directory {
            format!(" [{}]", dir)
        } else {
            String::new()
        };
        
        // Print progress line with carriage return (no newline)
        eprint!("\rProcessed {}/{} files (scanned: {}, skipped: {}){}", 
                processed, self.total_files, self.scanned_files, self.skipped_files, dir_info);
        io::stderr().flush().unwrap_or(());
    }

    /// Increment the scanned file count
    pub fn increment_scanned(&mut self) {
        if self.quiet_mode {
            return;
        }

        self.scanned_files += 1;
        
        // Update progress every 10 files or on final file to reduce flicker
        if self.scanned_files % 10 == 0 || self.scanned_files == self.total_files {
            self.update_progress();
        }
    }

    /// Increment the skipped file count (non-binary files)
    pub fn increment_skipped(&mut self) {
        if self.quiet_mode {
            return;
        }

        self.skipped_files += 1;
        
        // Update progress every 100 skipped files to reduce flicker
        if self.skipped_files % 100 == 0 {
            self.update_progress();
        }
    }

    /// Set the current directory being processed
    pub fn set_current_directory(&mut self, dir: &std::path::Path) {
        if self.quiet_mode {
            return;
        }
        
        // Get just the directory name, not the full path
        let dir_name = if let Some(name) = dir.file_name().and_then(|name| name.to_str()) {
            name.to_string()
        } else {
            dir.to_string_lossy().to_string()
        };
        
        self.current_directory = Some(dir_name);
        self.update_progress();
    }

    /// Complete the scanning process
    pub fn complete_scanning(&mut self) {
        if self.quiet_mode {
            return;
        }

        // Stop animation
        self.stop_current_animation();

        // Clear the line and show completion
        eprint!("\r");
        
        eprintln!("✓ Processed {}/{} files (scanned: {}, skipped: {}) - completed", 
                  self.scanned_files + self.skipped_files, 
                  self.total_files, 
                  self.scanned_files, 
                  self.skipped_files);
        
        io::stderr().flush().unwrap_or(());
    }

    /// Stop current animation thread
    fn stop_current_animation(&mut self) {
        if self.is_scanning.load(Ordering::Relaxed) {
            self.is_scanning.store(false, Ordering::Relaxed);
            
            if let Some(handle) = self.animation_handle.take() {
                let _ = handle.join();
            }
        }
    }

    /// Finish all scanning and clean up
    pub fn finish(&mut self) {
        self.stop_current_animation();
        
        if !self.quiet_mode {
            // Ensure we end with a clean line
            eprint!("\r");
            io::stderr().flush().unwrap_or(());
        }
    }

    /// Show a status message without animation (for errors, etc.)
    pub fn show_message(&self, message: &str) {
        if !self.quiet_mode {
            eprintln!("{}", message);
            io::stderr().flush().unwrap_or(());
        }
    }
}

impl Drop for ScanProgress {
    fn drop(&mut self) {
        self.finish();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_progress_creation() {
        let progress = ScanProgress::new();
        assert!(progress.current_directory.is_none());
        assert!(!progress.is_scanning.load(Ordering::Relaxed));
    }

    #[test]
    fn test_progress_operations() {
        let mut progress = ScanProgress::new();
        
        // Test setting current directory
        progress.set_current_directory(Path::new("/test"));
        assert!(progress.current_directory.is_some());
        
        // Test scanning operations
        progress.start_scanning(100);
        progress.increment_scanned();
        progress.increment_skipped();
        progress.complete_scanning();
        progress.finish();
    }
}