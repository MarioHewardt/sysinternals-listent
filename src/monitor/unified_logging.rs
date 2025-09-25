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