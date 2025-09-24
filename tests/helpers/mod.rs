use std::process::Command;
use std::path::{Path, PathBuf};
use std::fs;
use tempfile::TempDir;

pub mod reliable_runner;

/// Test helper for creating controlled test environments
pub struct TestEnvironment {
    pub temp_dir: TempDir,
    pub test_binaries: Vec<TestBinary>,
}

#[derive(Debug, Clone)]
pub struct TestBinary {
    pub name: String,
    pub path: PathBuf,
    pub expected_entitlements: Vec<String>,
}

impl TestEnvironment {
    /// Create a new controlled test environment
    pub fn new() -> anyhow::Result<Self> {
        let temp_dir = tempfile::tempdir()?;
        let mut env = TestEnvironment {
            temp_dir,
            test_binaries: Vec::new(),
        };
        
        // Create test binaries with known entitlements
        env.create_test_binaries()?;
        
        Ok(env)
    }
    
    /// Get the path to the test directory
    pub fn path(&self) -> &Path {
        self.temp_dir.path()
    }
    
    /// Create test binaries with predictable entitlements
    fn create_test_binaries(&mut self) -> anyhow::Result<()> {
        // Create a simple Swift program that we can sign with specific entitlements
        let swift_source = r#"
import Foundation
print("Test binary started with PID: \(getpid())")
// Keep running for a controllable amount of time
let args = CommandLine.arguments
if args.count > 1, let seconds = Double(args[1]) {
    Thread.sleep(forTimeInterval: seconds)
} else {
    Thread.sleep(forTimeInterval: 1.0)
}
print("Test binary exiting")
"#;
        
        // Create different test binaries with different entitlements
        let test_configs = vec![
            ("test_network", vec!["com.apple.security.network.client".to_string()]),
            ("test_debug", vec!["com.apple.security.get-task-allow".to_string()]),
            ("test_multi", vec![
                "com.apple.security.network.client".to_string(),
                "com.apple.security.network.server".to_string(),
            ]),
            ("test_no_entitlements", vec![]),
        ];
        
        for (name, entitlements) in test_configs {
            let binary_path = self.create_test_binary(name, swift_source, &entitlements)?;
            self.test_binaries.push(TestBinary {
                name: name.to_string(),
                path: binary_path,
                expected_entitlements: entitlements,
            });
        }
        
        Ok(())
    }
    
    /// Create a single test binary with specified entitlements
    fn create_test_binary(&self, name: &str, source: &str, entitlements: &[String]) -> anyhow::Result<PathBuf> {
        let source_path = self.temp_dir.path().join(format!("{}.swift", name));
        let binary_path = self.temp_dir.path().join(name);
        
        // Write Swift source
        fs::write(&source_path, source)?;
        
        // Compile Swift program
        let compile_result = Command::new("swiftc")
            .arg(&source_path)
            .arg("-o")
            .arg(&binary_path)
            .output()?;
            
        if !compile_result.status.success() {
            return Err(anyhow::anyhow!(
                "Failed to compile test binary {}: {}", 
                name, 
                String::from_utf8_lossy(&compile_result.stderr)
            ));
        }
        
        // Create entitlements plist if needed
        if !entitlements.is_empty() {
            let entitlements_plist = self.create_entitlements_plist(entitlements)?;
            let entitlements_path = self.temp_dir.path().join(format!("{}.entitlements", name));
            fs::write(&entitlements_path, entitlements_plist)?;
            
            // Sign the binary with entitlements
            let sign_result = Command::new("codesign")
                .arg("-s")
                .arg("-") // Ad-hoc signing
                .arg("--entitlements")
                .arg(&entitlements_path)
                .arg("-f") // Force
                .arg(&binary_path)
                .output()?;
                
            if !sign_result.status.success() {
                return Err(anyhow::anyhow!(
                    "Failed to sign test binary {}: {}", 
                    name, 
                    String::from_utf8_lossy(&sign_result.stderr)
                ));
            }
        } else {
            // Sign without entitlements (ad-hoc)
            let sign_result = Command::new("codesign")
                .arg("-s")
                .arg("-") // Ad-hoc signing
                .arg("-f") // Force
                .arg(&binary_path)
                .output()?;
                
            if !sign_result.status.success() {
                return Err(anyhow::anyhow!(
                    "Failed to sign test binary {}: {}", 
                    name, 
                    String::from_utf8_lossy(&sign_result.stderr)
                ));
            }
        }
        
        Ok(binary_path)
    }
    
    /// Create an entitlements plist for the given entitlements
    fn create_entitlements_plist(&self, entitlements: &[String]) -> anyhow::Result<String> {
        let mut plist = String::from(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
"#
        );
        
        for entitlement in entitlements {
            plist.push_str(&format!("    <key>{}</key>\n    <true/>\n", entitlement));
        }
        
        plist.push_str("</dict>\n</plist>\n");
        Ok(plist)
    }
    
    /// Spawn a test process that will run for the specified duration
    pub fn spawn_test_process(&self, binary_name: &str, duration_seconds: f64) -> anyhow::Result<std::process::Child> {
        let binary = self.test_binaries.iter()
            .find(|b| b.name == binary_name)
            .ok_or_else(|| anyhow::anyhow!("Test binary '{}' not found", binary_name))?;
            
        let child = Command::new(&binary.path)
            .arg(duration_seconds.to_string())
            .spawn()?;
            
        Ok(child)
    }
    
    /// Get expected entitlements for a test binary
    pub fn get_expected_entitlements(&self, binary_name: &str) -> Option<&Vec<String>> {
        self.test_binaries.iter()
            .find(|b| b.name == binary_name)
            .map(|b| &b.expected_entitlements)
    }
}

/// Test runner with timeout and cleanup
pub struct TestRunner {
    timeout_seconds: u64,
}

impl TestRunner {
    pub fn new(timeout_seconds: u64) -> Self {
        Self { timeout_seconds }
    }
    
    /// Run listent scan mode and capture output
    pub fn run_scan(&self, args: &[&str]) -> anyhow::Result<TestResult> {
        let start = std::time::Instant::now();
        
        let mut cmd = Command::new("./target/release/listent");
        for arg in args {
            cmd.arg(arg);
        }
        
        let output = cmd
            .timeout(std::time::Duration::from_secs(self.timeout_seconds))
            .output()?;
            
        Ok(TestResult {
            exit_code: output.status.code(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            duration: start.elapsed(),
        })
    }
    
    /// Run listent monitor mode and test CTRL-C
    pub fn run_monitor_with_interrupt(&self, args: &[&str], interrupt_after_seconds: f64) -> anyhow::Result<TestResult> {
        use std::process::Stdio;
        use std::time::Duration;
        
        let start = std::time::Instant::now();
        
        let mut cmd = Command::new("./target/release/listent");
        cmd.arg("--monitor");
        for arg in args {
            cmd.arg(arg);
        }
        
        let mut child = cmd
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
            
        // Wait for the specified duration, then send SIGINT
        std::thread::sleep(Duration::from_secs_f64(interrupt_after_seconds));
        
        // Send SIGINT (same as CTRL-C)
        unsafe {
            libc::kill(child.id() as i32, libc::SIGINT);
        }
        
        // Wait for process to exit (with timeout)
        let timeout = Duration::from_secs(5); // Give it 5 seconds to exit gracefully
        let result = wait_for_child_with_timeout(&mut child, timeout)?;
        
        Ok(TestResult {
            exit_code: result.status.code(),
            stdout: String::from_utf8_lossy(&result.stdout).to_string(),
            stderr: String::from_utf8_lossy(&result.stderr).to_string(),
            duration: start.elapsed(),
        })
    }
}

#[derive(Debug)]
pub struct TestResult {
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub duration: std::time::Duration,
}

impl TestResult {
    pub fn was_successful(&self) -> bool {
        self.exit_code == Some(0)
    }
    
    pub fn contains_stdout(&self, text: &str) -> bool {
        self.stdout.contains(text)
    }
    
    pub fn contains_stderr(&self, text: &str) -> bool {
        self.stderr.contains(text)
    }
}

/// Wait for a child process with timeout
fn wait_for_child_with_timeout(child: &mut std::process::Child, timeout: std::time::Duration) -> anyhow::Result<std::process::Output> {
    use std::sync::mpsc;
    use std::thread;
    
    let (tx, rx) = mpsc::channel();
    let child_id = child.id();
    
    // Spawn a thread to wait for the child
    thread::spawn(move || {
        // This is a simplified version - in production you'd want more robust process handling
        thread::sleep(timeout);
        tx.send(()).ok();
    });
    
    // Try to get output
    match child.wait_with_output() {
        Ok(output) => Ok(output),
        Err(e) => {
            // Kill the child if it's still running
            let _ = child.kill();
            Err(anyhow::anyhow!("Child process timed out: {}", e))
        }
    }
}

// Extension trait to add timeout to Command
trait CommandTimeout {
    fn timeout(&mut self, duration: std::time::Duration) -> &mut Self;
}

impl CommandTimeout for Command {
    fn timeout(&mut self, _duration: std::time::Duration) -> &mut Self {
        // This is a placeholder - for full implementation you'd want
        // to use a crate like `tokio` or implement custom timeout logic
        self
    }
}