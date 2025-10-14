#![allow(dead_code)]

use std::process::{Command, Child};
use std::time::{Duration, Instant};
use std::sync::mpsc;
use std::thread;
use anyhow::Result;

/// Test harness that ensures reliable cleanup and timeout handling
pub struct ReliableTestRunner {
    timeout: Duration,
    cleanup_handles: Vec<u32>,  // Just PIDs
}

impl ReliableTestRunner {
    pub fn new(timeout_seconds: u64) -> Self {
        Self {
            timeout: Duration::from_secs(timeout_seconds),
            cleanup_handles: Vec::new(),
        }
    }
    
    /// Run a command with automatic timeout and cleanup
    pub fn run_command_with_timeout(&mut self, mut cmd: Command) -> Result<TestOutput> {
        let start = Instant::now();
        
        // Ensure stdout and stderr are captured
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());
        
        let mut child = cmd.spawn()?;
        
        // Register for cleanup
        self.cleanup_handles.push(child.id());
        
        // Set up timeout mechanism
        let (tx, rx) = mpsc::channel();
        let timeout = self.timeout;
        
        // Spawn timeout thread
        thread::spawn(move || {
            thread::sleep(timeout);
            tx.send(()).ok();
        });
        
        // Wait for either completion or timeout
        let result = match child.try_wait() {
            Ok(Some(_status)) => {
                // Process already finished
                let output = child.wait_with_output()?;
                TestOutput::from_output(output, start.elapsed())
            },
            Ok(None) => {
                // Process still running, wait with timeout
                self.wait_with_timeout(child, rx, timeout)?
            },
            Err(e) => return Err(anyhow::anyhow!("Failed to check child process: {}", e)),
        };
        
        Ok(result)
    }
    
    /// Run listent in monitor mode with controlled interruption
    pub fn run_monitor_with_interrupt(&mut self, args: &[&str], interrupt_after: Duration) -> Result<TestOutput> {
        let mut cmd = Command::new("./target/release/listent");
        cmd.arg("--monitor");
        for arg in args {
            cmd.arg(arg);
        }
        
        // Ensure stdout and stderr are captured
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());
        
        let child = cmd.spawn()?;
        self.cleanup_handles.push(child.id());
        
        // Wait for specified duration
        thread::sleep(interrupt_after);
        
        // Send SIGINT
        self.send_sigint(child.id())?;
        
        // Wait for graceful shutdown (with timeout)
        let shutdown_timeout = Duration::from_secs(10);
        let (tx, rx) = mpsc::channel();
        
        thread::spawn(move || {
            thread::sleep(shutdown_timeout);
            tx.send(()).ok();
        });
        
        let result = self.wait_with_timeout(child, rx, shutdown_timeout)?;
        Ok(result)
    }
    
    /// Wait for child with timeout using channel signaling
    fn wait_with_timeout(&self, mut child: Child, timeout_rx: mpsc::Receiver<()>, _timeout: Duration) -> Result<TestOutput> {
        let start = Instant::now();
        
        // Use a more reliable approach with try_wait in a loop
        loop {
            match child.try_wait() {
                Ok(Some(_status)) => {
                    // Process finished
                    let output = child.wait_with_output()?;
                    return Ok(TestOutput::from_output(output, start.elapsed()));
                },
                Ok(None) => {
                    // Still running, check timeout
                    if timeout_rx.try_recv().is_ok() {
                        // Timeout reached, kill process
                        let _ = child.kill();
                        let output = child.wait_with_output()?;
                        return Ok(TestOutput::from_output_timeout(output, start.elapsed()));
                    }
                    // Sleep briefly before checking again
                    thread::sleep(Duration::from_millis(100));
                },
                Err(e) => return Err(anyhow::anyhow!("Error waiting for child: {}", e)),
            }
        }
    }
    
    /// Send SIGINT to a process
    fn send_sigint(&self, pid: u32) -> Result<()> {
        unsafe {
            let result = libc::kill(pid as i32, libc::SIGINT);
            if result != 0 {
                return Err(anyhow::anyhow!("Failed to send SIGINT to PID {}", pid));
            }
        }
        Ok(())
    }
    
    /// Kill a process forcefully
    fn kill_process(&self, pid: u32) -> Result<()> {
        unsafe {
            let result = libc::kill(pid as i32, libc::SIGKILL);
            if result != 0 {
                // Process might already be dead, which is fine
                return Ok(());
            }
        }
        Ok(())
    }
}

impl Drop for ReliableTestRunner {
    fn drop(&mut self) {
        // Clean up all registered resources
        for pid in &self.cleanup_handles {
            // Try graceful shutdown first, then force kill
            let _ = self.send_sigint(*pid);
            thread::sleep(Duration::from_millis(500));
            let _ = self.kill_process(*pid);
        }
    }
}

#[derive(Debug)]
pub struct TestOutput {
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub duration: Duration,
    pub timed_out: bool,
}

impl TestOutput {
    fn from_output(output: std::process::Output, duration: Duration) -> Self {
        Self {
            exit_code: output.status.code(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            duration,
            timed_out: false,
        }
    }
    
    fn from_output_timeout(output: std::process::Output, duration: Duration) -> Self {
        Self {
            exit_code: output.status.code(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            duration,
            timed_out: true,
        }
    }
    
    pub fn was_successful(&self) -> bool {
        !self.timed_out && self.exit_code == Some(0)
    }
    
    pub fn contains_stdout(&self, text: &str) -> bool {
        self.stdout.contains(text)
    }
    
    pub fn contains_stderr(&self, text: &str) -> bool {
        self.stderr.contains(text)
    }
    
    pub fn contains_output(&self, text: &str) -> bool {
        self.stdout.contains(text) || self.stderr.contains(text)
    }
}

/// Test scenario builder for complex integration tests
pub struct TestScenario {
    runner: ReliableTestRunner,
}

impl TestScenario {
    pub fn new(_name: &str, timeout_seconds: u64) -> Self {
        Self {
            runner: ReliableTestRunner::new(timeout_seconds),
        }
    }
    
    pub fn run_monitor_test(&mut self, monitor_args: &[&str], test_duration: Duration) -> Result<TestOutput> {
        self.runner.run_monitor_with_interrupt(monitor_args, test_duration)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_reliable_runner_timeout() -> Result<()> {
        let mut runner = ReliableTestRunner::new(2); // 2 second timeout
        
        // Run a command that should timeout (sleep longer than timeout)
        let mut cmd = Command::new("sleep");
        cmd.arg("10");
        let result = runner.run_command_with_timeout(cmd)?;
        
        assert!(result.timed_out, "Should have timed out");
        assert!(result.duration >= Duration::from_secs(2), "Should respect timeout");
        
        Ok(())
    }
    
    #[test]
    fn test_reliable_runner_success() -> Result<()> {
        let mut runner = ReliableTestRunner::new(5);
        
        let mut cmd = Command::new("echo");
        cmd.arg("hello");
        let result = runner.run_command_with_timeout(cmd)?;
        
        assert!(!result.timed_out, "Should not timeout");
        assert!(result.was_successful(), "Should succeed");
        assert!(result.contains_stdout("hello"), "Should contain expected output");
        
        Ok(())
    }
}