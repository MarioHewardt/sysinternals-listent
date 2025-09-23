//! Entitlement extraction module
//!
//! Handles:
//! - Extracting entitlements from Mach-O binaries using optimized plist parsing
//! - Fallback to manual XML parsing for compatibility
//! - Error handling for unsigned/malformed binaries
//! - Performance optimization for batch operations
//! - Pattern matching for entitlement filtering

use std::collections::HashMap;
use std::path::Path;
use std::process::Command;
use anyhow::{Result, anyhow};
use serde_json::Value;

pub mod pattern_matcher;
pub mod native;

/// Extract entitlements from a binary file
/// 
/// Uses optimized plist parsing for better performance,
/// with fallback to manual XML parsing if needed.
pub fn extract_entitlements(binary_path: &Path) -> Result<HashMap<String, Value>> {
    // Try optimized plist parsing first
    match native::extract_entitlements_optimized(binary_path) {
        Ok(entitlements) => return Ok(entitlements),
        Err(_) => {
            // Fall back to manual XML parsing if plist parsing fails
            // This provides compatibility for edge cases
        }
    }
    
    // Fallback to manual XML parsing (original implementation)
    extract_entitlements_codesign(binary_path)
}

/// Extract entitlements using codesign command-line tool (fallback method)
pub fn extract_entitlements_codesign(binary_path: &Path) -> Result<HashMap<String, Value>> {
    // Call codesign to extract entitlements
    let output = Command::new("codesign")
        .arg("-d")
        .arg("--entitlements")
        .arg("-")
        .arg("--xml")
        .arg(binary_path)
        .output()?;
    
    if !output.status.success() {
        // Binary might not be signed or might not have entitlements
        return Ok(HashMap::new());
    }
    
    let xml_content = String::from_utf8(output.stdout)?;
    
    // Parse the XML plist to extract entitlements
    parse_entitlements_plist(&xml_content)
}

/// Parse entitlements from XML plist format
fn parse_entitlements_plist(xml_content: &str) -> Result<HashMap<String, Value>> {
    // Simple XML parsing for plist format
    // Look for the main dict content between <dict> and </dict>
    
    if xml_content.trim().is_empty() {
        return Ok(HashMap::new());
    }
    
    // Find the main dictionary content
    let dict_start = xml_content.find("<dict>")
        .ok_or_else(|| anyhow!("No dict found in plist"))?;
    let dict_end = xml_content.rfind("</dict>")
        .ok_or_else(|| anyhow!("Unclosed dict in plist"))?;
    
    if dict_start >= dict_end {
        return Ok(HashMap::new());
    }
    
    let dict_content = &xml_content[dict_start + 6..dict_end];
    
    // Parse key-value pairs
    parse_plist_dict(dict_content)
}

/// Parse dictionary content from plist XML
fn parse_plist_dict(content: &str) -> Result<HashMap<String, Value>> {
    let mut entitlements = HashMap::new();
    let mut pos = 0;
    
    while pos < content.len() {
        // Find next <key> tag
        if let Some(key_start) = content[pos..].find("<key>") {
            let abs_key_start = pos + key_start + 5; // Skip "<key>"
            
            if let Some(key_end) = content[abs_key_start..].find("</key>") {
                let abs_key_end = abs_key_start + key_end;
                let key = content[abs_key_start..abs_key_end].trim().to_string();
                
                // Find the value after the key
                pos = abs_key_end + 6; // Skip "</key>"
                
                if let Some(value) = parse_next_plist_value(&content[pos..])? {
                    entitlements.insert(key, value.0);
                    pos += value.1;
                } else {
                    break;
                }
            } else {
                break;
            }
        } else {
            break;
        }
    }
    
    Ok(entitlements)
}

/// Parse the next value from plist XML
fn parse_next_plist_value(content: &str) -> Result<Option<(Value, usize)>> {
    let trimmed = content.trim_start();
    let offset = content.len() - trimmed.len();
    
    if trimmed.starts_with("<true/>") {
        Ok(Some((Value::Bool(true), offset + 7)))
    } else if trimmed.starts_with("<false/>") {
        Ok(Some((Value::Bool(false), offset + 8)))
    } else if trimmed.starts_with("<string>") {
        if let Some(end) = trimmed.find("</string>") {
            let value = trimmed[8..end].to_string();
            Ok(Some((Value::String(value), offset + end + 9)))
        } else {
            Ok(None)
        }
    } else if trimmed.starts_with("<integer>") {
        if let Some(end) = trimmed.find("</integer>") {
            let value_str = &trimmed[9..end];
            if let Ok(num) = value_str.parse::<i64>() {
                Ok(Some((Value::Number(num.into()), offset + end + 10)))
            } else {
                Ok(Some((Value::String(value_str.to_string()), offset + end + 10)))
            }
        } else {
            Ok(None)
        }
    } else if trimmed.starts_with("<array>") {
        // For simplicity, treat arrays as strings for now
        if let Some(end) = trimmed.find("</array>") {
            let array_content = &trimmed[7..end];
            Ok(Some((Value::String(format!("[array: {}]", array_content.trim())), offset + end + 8)))
        } else {
            Ok(None)
        }
    } else if trimmed.starts_with("<dict>") {
        // For simplicity, treat nested dicts as strings for now
        if let Some(end) = trimmed.find("</dict>") {
            let dict_content = &trimmed[6..end];
            Ok(Some((Value::String(format!("[dict: {}]", dict_content.trim())), offset + end + 7)))
        } else {
            Ok(None)
        }
    } else {
        // Skip unknown tags
        if let Some(tag_end) = trimmed.find('>') {
            Ok(Some((Value::String("[unknown]".to_string()), offset + tag_end + 1)))
        } else {
            Ok(None)
        }
    }
}