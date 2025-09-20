use crate::models::MonitoredProcess;
use anyhow::Result;

/// Initialize macOS Unified Logging
pub fn init_logger() -> Result<()> {
    // On macOS, initialize oslog
    #[cfg(target_os = "macos")]
    {
        // oslog will automatically use the bundle identifier as subsystem
        // We'll manually specify our subsystem in log calls
        Ok(())
    }
    
    #[cfg(not(target_os = "macos"))]
    {
        // For non-macOS platforms, logging is not available
        Err(anyhow::anyhow!("Unified Logging only available on macOS"))
    }
}

/// Log process detection event to Unified Logging System
pub fn log_process_detection(process: &MonitoredProcess) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        // For now, use a simple implementation until oslog API is clarified
        // Future: Integrate with proper oslog crate when API is stable
        let message = if process.entitlements.is_empty() {
            format!(
                "Process detected: {} (PID: {}) Path: {} Entitlements: []",
                process.name,
                process.pid,
                process.executable_path.display()
            )
        } else {
            format!(
                "Process detected: {} (PID: {}) Path: {} Entitlements: [{}]",
                process.name,
                process.pid,
                process.executable_path.display(),
                process.entitlements.join(", ")
            )
        };

        // Log to console for now - will be replaced with proper oslog
        eprintln!("[oslog] {}", message);
        Ok(())
    }
    
    #[cfg(not(target_os = "macos"))]
    {
        // Silently succeed on non-macOS platforms
        Ok(())
    }
}