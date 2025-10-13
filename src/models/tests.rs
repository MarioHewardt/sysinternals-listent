//! Unit tests for data models module
//!
//! Validates data structures, invariants, and validation rules defined
//! in the models module. Tests JSON serialization, filtering logic,
//! and business rule enforcement.

// Unit tests for data models - testing invariants and validation rules
// These tests validate the data structures defined in src/models/mod.rs

use super::*;
use std::collections::HashMap;
use serde_json;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_result_creation() {
        let mut entitlements = HashMap::new();
        entitlements.insert(
            "com.apple.security.network.client".to_string(),
            serde_json::Value::Bool(true)
        );

        let result = BinaryResult {
            path: "/Applications/Test.app/Contents/MacOS/Test".to_string(),
            entitlements: entitlements.clone(),
            entitlement_count: entitlements.len(),
        };

        assert_eq!(result.path, "/Applications/Test.app/Contents/MacOS/Test");
        assert_eq!(result.entitlement_count, 1);
        assert_eq!(result.entitlements.len(), result.entitlement_count);
    }

    #[test]
    fn test_scan_summary_creation() {
        let summary = ScanSummary {
            scanned: 100,
            matched: 25,
            skipped_unreadable: 5,
            duration_ms: 1500,
            interrupted: None,
        };

        assert_eq!(summary.scanned, 100);
        assert_eq!(summary.matched, 25);
        assert_eq!(summary.skipped_unreadable, 5);
        assert_eq!(summary.duration_ms, 1500);
        assert!(summary.interrupted.is_none());
    }

    #[test]
    fn test_scan_summary_interrupted_serialization() {
        // Test that interrupted = None is omitted in JSON
        let summary_not_interrupted = ScanSummary {
            scanned: 10,
            matched: 5,
            skipped_unreadable: 0,
            duration_ms: 500,
            interrupted: None,
        };

        let json = serde_json::to_string(&summary_not_interrupted).unwrap();
        assert!(!json.contains("interrupted"));

        // Test that interrupted = Some(true) is included
        let summary_interrupted = ScanSummary {
            scanned: 10,
            matched: 5,
            skipped_unreadable: 0,
            duration_ms: 500,
            interrupted: Some(true),
        };

        let json = serde_json::to_string(&summary_interrupted).unwrap();
        assert!(json.contains("\"interrupted\":true"));
    }

    #[test]
    fn test_entitlement_scan_output_structure() {
        let mut entitlements = HashMap::new();
        entitlements.insert("test.entitlement".to_string(), serde_json::Value::Bool(true));

        let result = BinaryResult {
            path: "/test/path".to_string(),
            entitlements,
            entitlement_count: 1,
        };

        let summary = ScanSummary {
            scanned: 1,
            matched: 1,
            skipped_unreadable: 0,
            duration_ms: 100,
            interrupted: None,
        };

        let output = EntitlementScanOutput {
            results: vec![result],
            summary,
        };

        assert_eq!(output.results.len(), 1);
        assert_eq!(output.summary.scanned, 1);
        assert_eq!(output.summary.matched, 1);
    }

    #[test]
    fn test_scan_config_defaults() {
        let filters = ScanFilters::default();
        assert!(filters.entitlements.is_empty());
    }

    #[test]
    fn test_binary_result_entitlement_count_consistency() {
        // Test that entitlement_count matches actual entitlements size
        let mut entitlements = HashMap::new();
        entitlements.insert("ent1".to_string(), serde_json::Value::Bool(true));
        entitlements.insert("ent2".to_string(), serde_json::Value::String("test".to_string()));

        let result = BinaryResult {
            path: "/test".to_string(),
            entitlements: entitlements.clone(),
            entitlement_count: entitlements.len(),
        };

        assert_eq!(result.entitlement_count, result.entitlements.len());
        assert_eq!(result.entitlement_count, 2);
    }

    #[test]
    fn test_invalid_interval_error_message() {
        let error = invalid_interval_error(500.0);
        let error_msg = error.to_string();
        
        assert!(error_msg.contains("Invalid polling interval: 500"));
        assert!(error_msg.contains("Must be between"));
        assert!(error_msg.contains("0.1"));
        assert!(error_msg.contains("300"));
    }

    #[test]
    fn test_invalid_interval_error_boundary_values() {
        // Test below minimum
        let error = invalid_interval_error(0.05);
        assert!(error.to_string().contains("0.05"));
        
        // Test above maximum
        let error = invalid_interval_error(301.0);
        assert!(error.to_string().contains("301"));
    }
}