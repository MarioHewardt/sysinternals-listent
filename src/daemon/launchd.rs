//! macOS LaunchD integration for daemon service management
//!
//! Handles plist generation, service installation, and lifecycle management

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;
use crate::constants::{LAUNCHD_SERVICE_NAME, LAUNCHD_PLIST_NAME};

/// LaunchD plist configuration
#[derive(Debug, Clone)]
pub struct LaunchDPlist {
    /// Service label (reverse DNS format)
    pub label: String,
    /// Executable path and arguments
    pub program_arguments: Vec<String>,
    /// Whether to start at boot/login
    pub run_at_load: bool,
    /// Whether to restart if process exits
    pub keep_alive: bool,
    /// Working directory for daemon
    pub working_directory: Option<PathBuf>,
    /// Standard output log file
    pub standard_out_path: Option<PathBuf>,
    /// Standard error log file
    pub standard_error_path: Option<PathBuf>,
    /// Environment variables
    pub environment_variables: Option<std::collections::HashMap<String, String>>,
}

impl LaunchDPlist {
    /// Create a new LaunchD plist with default settings
    pub fn new(daemon_path: &Path) -> Self {
        Self {
            label: LAUNCHD_SERVICE_NAME.to_string(),
            program_arguments: vec![
                daemon_path.to_string_lossy().to_string(),
                "--daemon".to_string(),
            ],
            run_at_load: true,
            keep_alive: true,
            working_directory: Some(PathBuf::from("/var/run/listent")),
            standard_out_path: Some(PathBuf::from("/var/log/listent/daemon.log")),
            standard_error_path: Some(PathBuf::from("/var/log/listent/daemon.log")),
            environment_variables: Some({
                let mut env = std::collections::HashMap::new();
                env.insert("PATH".to_string(), "/usr/bin:/bin:/usr/sbin:/sbin".to_string());
                env.insert("LISTENT_DAEMON_CHILD".to_string(), "1".to_string());
                env
            }),
        }
    }

    /// Generate plist XML content
    pub fn generate_plist(&self) -> Result<String> {
        let mut plist = String::new();
        
        plist.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        plist.push_str("<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n");
        plist.push_str("<plist version=\"1.0\">\n");
        plist.push_str("<dict>\n");

        // Label
        plist.push_str("\t<key>Label</key>\n");
        plist.push_str(&format!("\t<string>{}</string>\n", self.label));

        // Program arguments
        plist.push_str("\t<key>ProgramArguments</key>\n");
        plist.push_str("\t<array>\n");
        for arg in &self.program_arguments {
            plist.push_str(&format!("\t\t<string>{}</string>\n", arg));
        }
        plist.push_str("\t</array>\n");

        // RunAtLoad
        plist.push_str("\t<key>RunAtLoad</key>\n");
        plist.push_str(&format!("\t<{}/>\n", if self.run_at_load { "true" } else { "false" }));

        // KeepAlive
        plist.push_str("\t<key>KeepAlive</key>\n");
        plist.push_str(&format!("\t<{}/>\n", if self.keep_alive { "true" } else { "false" }));

        // Working directory
        if let Some(ref working_dir) = self.working_directory {
            plist.push_str("\t<key>WorkingDirectory</key>\n");
            plist.push_str(&format!("\t<string>{}</string>\n", working_dir.display()));
        }

        // Standard output
        if let Some(ref stdout_path) = self.standard_out_path {
            plist.push_str("\t<key>StandardOutPath</key>\n");
            plist.push_str(&format!("\t<string>{}</string>\n", stdout_path.display()));
        }

        // Standard error
        if let Some(ref stderr_path) = self.standard_error_path {
            plist.push_str("\t<key>StandardErrorPath</key>\n");
            plist.push_str(&format!("\t<string>{}</string>\n", stderr_path.display()));
        }

        // Environment variables
        if let Some(ref env_vars) = self.environment_variables {
            if !env_vars.is_empty() {
                plist.push_str("\t<key>EnvironmentVariables</key>\n");
                plist.push_str("\t<dict>\n");
                for (key, value) in env_vars {
                    plist.push_str(&format!("\t\t<key>{}</key>\n", key));
                    plist.push_str(&format!("\t\t<string>{}</string>\n", value));
                }
                plist.push_str("\t</dict>\n");
            }
        }

        plist.push_str("</dict>\n");
        plist.push_str("</plist>\n");

        Ok(plist)
    }

    /// Install plist file to appropriate location
    fn install_plist(&self, plist_content: &str) -> Result<PathBuf> {
        // Use LaunchDaemons directory for system-wide service (requires sudo)
        let plist_path = Path::new("/Library/LaunchDaemons")
            .join(LAUNCHD_PLIST_NAME);

        // Write plist file
        std::fs::write(&plist_path, plist_content)
            .with_context(|| format!("Failed to write plist file: {}", plist_path.display()))?;

        Ok(plist_path)
    }

    /// Load service with launchctl
    pub fn launchctl_load(&self, plist_path: &Path) -> Result<()> {
        let output = Command::new("launchctl")
            .args(&["load", plist_path.to_str().unwrap()])
            .output()
            .context("Failed to execute launchctl load")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("launchctl load failed: {}", stderr);
        }

        Ok(())
    }

    /// Install daemon service to LaunchD (minimal version)
    pub fn install_service(&self, _daemon_path: &std::path::Path, _config_path: Option<&std::path::Path>) -> Result<()> {
        let plist_content = self.generate_plist()?;
        let plist_path = self.install_plist(&plist_content)?;
        self.launchctl_load(&plist_path)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_launchd_plist_creation() {
        let daemon_path = Path::new("/usr/local/bin/listent");
        let plist = LaunchDPlist::new(daemon_path);
        
        assert_eq!(plist.label, "com.microsoft.sysinternals.listent");
        assert_eq!(plist.program_arguments.len(), 2);
        assert_eq!(plist.program_arguments[0], "/usr/local/bin/listent");
        assert_eq!(plist.program_arguments[1], "--daemon");
        assert!(plist.run_at_load);
        assert!(plist.keep_alive);
    }

    #[test]
    fn test_plist_xml_structure() {
        let daemon_path = Path::new("/usr/local/bin/listent");
        let plist = LaunchDPlist::new(daemon_path);
        let xml = plist.generate_plist().unwrap();
        
        // Check XML structure
        assert!(xml.contains("<?xml version=\"1.0\""));
        assert!(xml.contains("<!DOCTYPE plist"));
        assert!(xml.contains("<plist version=\"1.0\">"));
        assert!(xml.contains("</plist>"));
    }

    #[test]
    fn test_plist_contains_label() {
        let daemon_path = Path::new("/usr/local/bin/listent");
        let plist = LaunchDPlist::new(daemon_path);
        let xml = plist.generate_plist().unwrap();
        
        assert!(xml.contains("<key>Label</key>"));
        assert!(xml.contains("<string>com.microsoft.sysinternals.listent</string>"));
    }

    #[test]
    fn test_plist_contains_program_arguments() {
        let daemon_path = Path::new("/usr/local/bin/listent");
        let plist = LaunchDPlist::new(daemon_path);
        let xml = plist.generate_plist().unwrap();
        
        assert!(xml.contains("<key>ProgramArguments</key>"));
        assert!(xml.contains("<array>"));
        assert!(xml.contains("<string>/usr/local/bin/listent</string>"));
        assert!(xml.contains("<string>--daemon</string>"));
    }

    #[test]
    fn test_plist_contains_run_at_load() {
        let daemon_path = Path::new("/usr/local/bin/listent");
        let plist = LaunchDPlist::new(daemon_path);
        let xml = plist.generate_plist().unwrap();
        
        assert!(xml.contains("<key>RunAtLoad</key>"));
        assert!(xml.contains("<true/>"));
    }

    #[test]
    fn test_plist_contains_keep_alive() {
        let daemon_path = Path::new("/usr/local/bin/listent");
        let plist = LaunchDPlist::new(daemon_path);
        let xml = plist.generate_plist().unwrap();
        
        assert!(xml.contains("<key>KeepAlive</key>"));
        assert!(xml.contains("<true/>"));
    }

    #[test]
    fn test_plist_contains_working_directory() {
        let daemon_path = Path::new("/usr/local/bin/listent");
        let plist = LaunchDPlist::new(daemon_path);
        let xml = plist.generate_plist().unwrap();
        
        assert!(xml.contains("<key>WorkingDirectory</key>"));
        assert!(xml.contains("<string>/var/run/listent</string>"));
    }

    #[test]
    fn test_plist_contains_log_paths() {
        let daemon_path = Path::new("/usr/local/bin/listent");
        let plist = LaunchDPlist::new(daemon_path);
        let xml = plist.generate_plist().unwrap();
        
        assert!(xml.contains("<key>StandardOutPath</key>"));
        assert!(xml.contains("<string>/var/log/listent/daemon.log</string>"));
        assert!(xml.contains("<key>StandardErrorPath</key>"));
    }

    #[test]
    fn test_plist_contains_environment_variables() {
        let daemon_path = Path::new("/usr/local/bin/listent");
        let plist = LaunchDPlist::new(daemon_path);
        let xml = plist.generate_plist().unwrap();
        
        assert!(xml.contains("<key>EnvironmentVariables</key>"));
        assert!(xml.contains("<dict>"));
        assert!(xml.contains("<key>PATH</key>"));
        assert!(xml.contains("<string>/usr/bin:/bin:/usr/sbin:/sbin</string>"));
        assert!(xml.contains("<key>LISTENT_DAEMON_CHILD</key>"));
        assert!(xml.contains("<string>1</string>"));
    }

    #[test]
    fn test_plist_xml_well_formed() {
        let daemon_path = Path::new("/usr/local/bin/listent");
        let plist = LaunchDPlist::new(daemon_path);
        let xml = plist.generate_plist().unwrap();
        
        // Check balanced tags
        let opening_tags = xml.matches("<dict>").count();
        let closing_tags = xml.matches("</dict>").count();
        assert_eq!(opening_tags, closing_tags, "dict tags should be balanced");
        
        let array_open = xml.matches("<array>").count();
        let array_close = xml.matches("</array>").count();
        assert_eq!(array_open, array_close, "array tags should be balanced");
    }

    #[test]
    fn test_plist_custom_daemon_path() {
        let daemon_path = Path::new("/custom/path/to/daemon");
        let plist = LaunchDPlist::new(daemon_path);
        let xml = plist.generate_plist().unwrap();
        
        assert!(xml.contains("<string>/custom/path/to/daemon</string>"));
    }

    #[test]
    fn test_plist_without_environment_variables() {
        let daemon_path = Path::new("/usr/local/bin/listent");
        let mut plist = LaunchDPlist::new(daemon_path);
        plist.environment_variables = None;
        
        let xml = plist.generate_plist().unwrap();
        assert!(!xml.contains("<key>EnvironmentVariables</key>"));
    }

    #[test]
    fn test_plist_without_working_directory() {
        let daemon_path = Path::new("/usr/local/bin/listent");
        let mut plist = LaunchDPlist::new(daemon_path);
        plist.working_directory = None;
        
        let xml = plist.generate_plist().unwrap();
        assert!(!xml.contains("<key>WorkingDirectory</key>"));
    }

    #[test]
    fn test_plist_run_at_load_false() {
        let daemon_path = Path::new("/usr/local/bin/listent");
        let mut plist = LaunchDPlist::new(daemon_path);
        plist.run_at_load = false;
        
        let xml = plist.generate_plist().unwrap();
        assert!(xml.contains("<key>RunAtLoad</key>"));
        assert!(xml.contains("<false/>"));
    }

    #[test]
    fn test_plist_keep_alive_false() {
        let daemon_path = Path::new("/usr/local/bin/listent");
        let mut plist = LaunchDPlist::new(daemon_path);
        plist.keep_alive = false;
        
        let xml = plist.generate_plist().unwrap();
        assert!(xml.contains("<key>KeepAlive</key>"));
        assert!(xml.contains("<false/>"));
    }
}
