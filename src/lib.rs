//! listent library - macOS entitlement scanning utilities
//!
//! This library provides core functionality for scanning and monitoring
//! macOS code signing entitlements.

#![forbid(unsafe_code)]

pub mod models;
pub mod entitlements;
pub mod scan;
pub mod output;
pub mod monitor;
pub mod daemon;
pub mod constants;
pub mod cli;