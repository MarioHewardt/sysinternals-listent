//! Real-time process monitoring module
//!
//! Provides functionality for monitoring newly spawned processes and extracting
//! their entitlements in real-time. Supports configurable polling intervals
//! and filtering options.
//!
//! ## Key Components
//! - `ProcessTracker`: State management for tracking process changes
//! - `polling`: Main monitoring loop with interrupt handling
//! - `unified_logging`: macOS ULS integration for daemon logging

pub mod process_tracker;
pub mod polling;
pub mod unified_logging;

pub use process_tracker::ProcessTracker;
pub use unified_logging::init_logger;