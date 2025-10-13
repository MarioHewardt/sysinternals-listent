# listent - Test Specifications

> **Last Updated**: September 26, 2025  
> **Auto-generated from**: Test suite analysis

This document provides a comprehensive overview of all test scenarios in the listent project, organized by functional areas and test types.

## Test Organization

The test suite is organized into several categories:

- **Unit Tests**: Located in `src/` modules (`#[cfg(test)]`)
- **Contract Tests**: Located in `tests/contract/` - Validate CLI contracts and output formats
- **Integration Tests**: Located in `tests/integration/` - Test component interactions
- **Functional Tests**: Located in `tests/` root - End-to-end workflows
- **Helper Infrastructure**: Located in `tests/helpers/` - Test utilities and reliable runners

---

## 1. Core Functionality Tests

### Static Scanning Tests (`functional_static_scan.rs`)

| Test Name | Scenario | Verification |
|-----------|----------|--------------|
| `test_static_scan_with_controlled_binaries` | Scans known test binaries with expected entitlements | Validates entitlement extraction accuracy |
| `test_static_scan_with_entitlement_filter` | Filters scan results by specific entitlement patterns | Confirms filtering logic works correctly |
| `test_static_scan_human_readable_output` | Produces human-readable formatted output | Verifies output formatting and readability |
| `test_static_scan_interrupt_handling` | Handles CTRL-C interruption gracefully | Ensures clean shutdown on signal |
| `test_nonexistent_path_handling` | Handles non-existent directory paths | Returns appropriate error messages |
| `test_empty_directory_scan` | Scans empty directories without errors | Handles edge case gracefully |

### Process Monitoring Tests (`functional_monitor.rs`)

| Test Name | Scenario | Verification |
|-----------|----------|--------------|
| `test_monitor_mode_basic_functionality` | Starts monitoring mode successfully | Basic monitor startup and operation |
| `test_monitor_mode_detects_new_processes` | Detects newly spawned processes | Process detection capability |
| `test_monitor_mode_entitlement_filtering` | Filters monitored processes by entitlements | Entitlement-based process filtering |
| `test_monitor_mode_ctrl_c_handling` | Handles CTRL-C interruption in monitor mode | Clean shutdown during monitoring |
| `test_monitor_mode_different_intervals` | Tests various polling intervals (0.5s, 1.0s, 2.0s) | Interval parameter validation |
| `test_monitor_mode_invalid_interval` | Rejects invalid polling intervals (< 0.1s) | Input validation and error handling |
| `test_monitor_mode_with_path_filters` | Filters monitored processes by executable path | Path-based process filtering |
| `test_monitor_mode_json_output_format` | Outputs process detections in JSON format | JSON output formatting |

---

## 2. CLI Interface Tests

### Command Line Argument Tests (`test_monitor_cli.rs`)

| Test Name | Scenario | Verification |
|-----------|----------|--------------|
| `test_monitor_flag_parsing` | Parses --monitor flag correctly | CLI argument parsing |
| `test_interval_parameter_validation` | Validates --interval parameter values | Parameter validation logic |
| `test_monitor_help_text` | Displays helpful monitor mode documentation | Help text completeness |
| `test_monitor_with_invalid_arguments` | Rejects invalid argument combinations | Error handling for bad inputs |

### Help and Version Tests (`test_cli_help.rs`, `test_cli_version.rs`)

| Test Name | Scenario | Verification |
|-----------|----------|--------------|
| `test_help_includes_required_options` | Help text includes all required options | Documentation completeness |
| `test_help_describes_path_option` | Help explains path filtering option | Option documentation |
| `test_help_describes_entitlement_filter` | Help explains entitlement filtering | Filter documentation |
| `test_version_prints_semantic_version` | Version output follows semantic versioning | Version format compliance |
| `test_version_includes_commit_hash` | Version includes git commit information | Build traceability |
| `test_short_version_flag` | Short -V flag works correctly | CLI convenience |

---

## 3. Output Format Tests

### JSON Output Tests (`test_json_output.rs`)

| Test Name | Scenario | Verification |
|-----------|----------|--------------|
| `test_json_output_flag` | --json flag produces valid JSON | JSON format compliance |
| `test_json_output_is_valid_json` | Output parses as valid JSON | JSON validity |
| `test_json_output_deterministic_ordering` | JSON output order is consistent | Deterministic output |
| `test_json_output_with_filters` | JSON output respects filtering options | Filter integration |
| `test_json_vs_human_output_different` | JSON and human outputs differ appropriately | Format differentiation |

### Human-Readable Output Tests (`test_monitor_output.rs`)

| Test Name | Scenario | Verification |
|-----------|----------|--------------|
| `test_human_readable_output_format` | Human output is properly formatted | Readability and structure |
| `test_json_output_format` | JSON output structure is correct | JSON schema compliance |
| `test_timestamp_formatting` | Timestamps use ISO 8601 format | Time format standardization |
| `test_entitlements_list_formatting` | Entitlement lists are clearly formatted | List presentation |
| `test_quiet_mode_output_suppression` | --quiet suppresses non-essential output | Quiet mode behavior |
| `test_no_entitlements_formatting` | Handles binaries with no entitlements | Edge case handling |

---

## 4. Pattern Matching and Filtering Tests

### Entitlement Pattern Tests (`test_pattern_matching.rs`)

| Test Name | Scenario | Verification |
|-----------|----------|--------------|
| `test_exact_entitlement_filter_backwards_compatibility` | Exact string matching works | Backward compatibility |
| `test_exact_filter_no_substring_matching` | Exact filters don't match substrings | Precision in filtering |
| `test_glob_pattern_wildcard` | Glob patterns with * work correctly | Wildcard functionality |
| `test_glob_pattern_any_network` | Network-related glob patterns | Domain-specific patterns |
| `test_multiple_glob_patterns` | Multiple patterns combine correctly | Pattern combination |
| `test_invalid_glob_pattern_validation` | Invalid patterns are rejected | Input validation |
| `test_monitor_mode_glob_patterns` | Glob patterns work in monitor mode | Monitor integration |
| `test_monitor_mode_consistent_exact_matching` | Exact matching works in monitor mode | Consistency across modes |
| `test_json_output_with_patterns` | Pattern filtering works with JSON output | JSON integration |
| `test_comma_separated_patterns` | Multiple patterns via comma separation | CLI convenience |
| `test_pattern_case_sensitivity` | Case sensitivity handling | Pattern matching precision |

---

## 5. Daemon and LaunchD Integration Tests

### Daemon CLI Tests (`test_daemon_cli.rs`)

| Test Name | Scenario | Verification |
|-----------|----------|--------------|
| `test_install_daemon_subcommand` | install-daemon command works | Daemon installation |
| `test_install_daemon_with_config` | install-daemon accepts configuration | Config file handling |
| `test_uninstall_daemon_subcommand` | uninstall-daemon command works | Daemon removal |
| `test_daemon_status_subcommand` | daemon-status command works | Status reporting |
| `test_update_config_subcommand` | update-config command works | Runtime config updates |
| `test_logs_subcommand` | logs command works | Log access |
| `test_logs_with_follow` | logs --follow works | Live log streaming |
| `test_logs_with_since` | logs --since filtering works | Time-based log filtering |
| `test_daemon_flag_compatibility` | --daemon flag compatibility | Flag interactions |
| `test_help_shows_daemon_subcommands` | Help shows daemon commands | Documentation |

### LaunchD Integration Tests (`test_launchd_integration.rs`)

| Test Name | Scenario | Verification |
|-----------|----------|--------------|
| `test_launchd_flag_requires_daemon_mode` | --launchd requires daemon mode | Flag dependency validation |
| `test_launchd_permission_check` | LaunchD permission validation | Security checks |
| `test_launchd_help_message` | LaunchD help information | User guidance |
| `test_launchd_with_custom_arguments` | LaunchD with custom daemon args | Argument passing |
| `test_launchd_installation_output` | Installation success feedback | User feedback |

### Daemon Integration Tests (`test_daemon_integration.rs`)

| Test Name | Scenario | Verification |
|-----------|----------|--------------|
| `test_daemon_startup_with_args` | Daemon starts with correct arguments | Argument processing |
| `test_daemon_missing_arguments` | Handles missing required arguments | Error handling |
| `test_daemon_configuration_validation` | Validates daemon configuration | Config validation |
| `test_daemon_consistent_cli_args` | CLI args match daemon behavior | Consistency |
| `test_daemon_interval_argument` | Interval argument processing | Parameter handling |

---

## 6. Integration Tests

### Monitor Path Filtering (`integration/test_monitor_path_filtering.rs`)

| Test Name | Scenario | Verification |
|-----------|----------|--------------|
| `test_monitor_with_single_path_filter` | Single path filter in monitor mode | Path filtering |
| `test_monitor_with_multiple_path_filters` | Multiple path filters combine correctly | Multi-path filtering |
| `test_monitor_with_nonexistent_path` | Handles non-existent paths gracefully | Error handling |
| `test_path_filtering_effectiveness` | Path filters actually work | Filter effectiveness |
| `test_monitor_system_applications_path` | System /Applications path filtering | Common use case |
| `test_path_filter_with_json_output` | Path filtering with JSON output | Output integration |
| `test_path_filter_validation` | Path filter validation | Input validation |

### Monitor Entitlement Filtering (`integration/test_monitor_entitlement_filtering.rs`)

| Test Name | Scenario | Verification |
|-----------|----------|--------------|
| `test_monitor_with_entitlement_filter` | Entitlement filtering in monitor mode | Entitlement-based filtering |
| `test_monitor_with_multiple_entitlement_filters` | Multiple entitlement filters | Multi-filter support |
| `test_entitlement_filter_effectiveness` | Filters actually reduce results | Filter effectiveness |
| `test_monitor_entitlement_glob_patterns` | Glob patterns in monitor mode | Pattern integration |

---

## 7. Contract Tests (API/CLI Contracts)

### CLI Argument Validation (`contract/test_cli_args_validation.rs`)

| Test Name | Scenario | Verification |
|-----------|----------|--------------|
| `test_invalid_path_returns_error` | Invalid paths return error codes | Error handling |
| `test_path_not_directory_returns_error` | Non-directory paths return errors | Path validation |
| `test_daemon_and_launchd_combination` | --daemon and --launchd work together | Flag combinations |
| `test_duplicate_entitlements_accepted` | Duplicate entitlement filters accepted | Input tolerance |

### JSON Schema Validation (`contract/test_json_schema.rs`)

| Test Name | Scenario | Verification |
|-----------|----------|--------------|
| `test_json_output_structure` | JSON output has required structure | Schema compliance |
| `test_json_result_entry_structure` | Individual result entries are correct | Entry format |
| `test_json_no_extra_fields` | No unexpected fields in JSON | Schema precision |

### Human Output Format (`contract/test_human_output_format.rs`)

| Test Name | Scenario | Verification |
|-----------|----------|--------------|
| `test_human_output_format_structure` | Human output follows expected format | Format consistency |
| `test_human_output_summary_format` | Summary section formatting | Summary structure |
| `test_human_output_no_matches_case` | "No matches" case handling | Edge case formatting |
| `test_entitlement_line_format` | Individual entitlement formatting | Line format |
| `test_quiet_mode_suppresses_warnings` | Quiet mode suppresses warnings | Quiet behavior |

### Monitor Filters (`contract/test_monitor_filters.rs`)

| Test Name | Scenario | Verification |
|-----------|----------|--------------|
| `test_monitor_with_path_filter` | Path filters work in monitor mode | Monitor path filtering |
| `test_monitor_with_entitlement_filter` | Entitlement filters work in monitor | Monitor entitlement filtering |
| `test_monitor_with_json_output` | JSON output in monitor mode | Monitor JSON output |
| `test_monitor_with_quiet_mode` | Quiet mode in monitor | Monitor quiet behavior |
| `test_monitor_with_multiple_path_filters` | Multiple paths in monitor | Monitor multi-path |
| `test_monitor_with_multiple_entitlement_filters` | Multiple entitlements in monitor | Monitor multi-entitlement |
| `test_monitor_combined_filters` | Path and entitlement filters together | Monitor filter combination |

### Unified Logging System (`contract/test_unified_logging.rs`)

| Test Name | Scenario | Verification |
|-----------|----------|--------------|
| `test_unified_logging_integration` | ULS integration works | Logging integration |
| `test_log_subsystem_and_category` | Correct subsystem and category | Log categorization |
| `test_log_message_format` | Log message formatting | Message structure |
| `test_graceful_degradation_when_logging_unavailable` | Handles ULS unavailability | Graceful degradation |
| `test_logging_with_process_detection` | Logs process detection events | Event logging |
| `test_structured_logging_metadata` | Structured metadata in logs | Log metadata |

---

## 8. Comprehensive End-to-End Tests

### Comprehensive Functional Tests (`functional_comprehensive.rs`)

| Test Name | Scenario | Verification |
|-----------|----------|--------------|
| `test_end_to_end_static_scan_workflow` | Complete static scan workflow | Full scan pipeline |
| `test_end_to_end_monitor_workflow` | Complete monitor workflow | Full monitor pipeline |
| `test_signal_handling_reliability` | CTRL-C handling across modes | Signal handling robustness |
| `test_process_detection_with_controlled_processes` | Process detection with known processes | Detection accuracy |
| `test_error_handling_and_edge_cases` | Various error conditions | Error resilience |
| `test_output_format_consistency` | Output consistency across modes | Format reliability |
| `test_performance_and_timeout_handling` | Performance under load | Performance characteristics |
| `test_concurrent_operations` | Multiple operations simultaneously | Concurrency safety |
| `test_long_running_monitor_stability` | Extended monitor operation | Long-term stability |

### Simple Functional Tests (`simple_functional.rs`)

| Test Name | Scenario | Verification |
|-----------|----------|--------------|
| `test_basic_scan_functionality` | Basic scanning works | Core functionality |
| `test_basic_monitor_startup_and_shutdown` | Monitor starts and stops | Monitor lifecycle |
| `test_help_and_version_flags` | Help and version flags work | Basic CLI |
| `test_empty_directory_scan` | Empty directory handling | Edge case |
| `test_nonexistent_path_handling` | Non-existent path handling | Error case |
| `test_json_vs_human_output` | Output format differences | Format validation |
| `test_entitlement_filtering` | Basic entitlement filtering | Filter functionality |

---

## 9. Unit Tests (Internal Components)

### Entitlement Pattern Matching (`src/entitlements/pattern_matcher.rs`)

| Test Name | Scenario | Verification |
|-----------|----------|--------------|
| `test_is_glob_pattern` | Detects glob patterns correctly | Pattern recognition |
| `test_exact_matching` | Exact string matching | Exact match logic |
| `test_glob_pattern_matching` | Glob pattern matching | Wildcard logic |
| `test_entitlements_match_filters` | Filter matching logic | Core filtering |

### Entitlement Extraction (`src/entitlements/native.rs`)

| Test Name | Scenario | Verification |
|-----------|----------|--------------|
| `test_optimized_extraction_system_binary` | System binary entitlement extraction | Native extraction |
| `test_optimized_extraction_unsigned_binary` | Unsigned binary handling | Edge case handling |

### Progress Display (`src/output/progress.rs`)

| Test Name | Scenario | Verification |
|-----------|----------|--------------|
| `test_progress_creation` | Progress indicator creation | UI component |
| `test_progress_operations` | Progress indicator updates | UI updates |

### Test Infrastructure (`tests/helpers/reliable_runner.rs`)

| Test Name | Scenario | Verification |
|-----------|----------|--------------|
| `test_reliable_runner_timeout` | Timeout handling in test runner | Test infrastructure |
| `test_reliable_runner_success` | Successful execution handling | Test reliability |

---

## Test Categories Summary

| Category | Count | Purpose |
|----------|-------|---------|
| **Functional Tests** | 24 | End-to-end workflows and user scenarios |
| **CLI Interface Tests** | 13 | Command-line argument parsing and validation |
| **Output Format Tests** | 12 | JSON and human-readable output verification |
| **Pattern Matching Tests** | 11 | Entitlement filtering and glob patterns |
| **Integration Tests** | 15 | Component interaction verification |
| **Contract Tests** | 25 | API/CLI contract validation |
| **Daemon Tests** | 15 | Background daemon functionality |
| **Unit Tests** | 8 | Internal component testing |
| **Test Infrastructure** | 2 | Test tooling and reliability |

**Total Test Count: ~125 tests**

---

## Maintenance Notes

This document is automatically updated when tests are added, removed, or modified. Key areas to update:

1. **New Test Files**: Add new test categories as needed
2. **Modified Tests**: Update test descriptions when behavior changes
3. **Test Organization**: Reorganize categories as the codebase evolves
4. **Coverage Gaps**: Identify and document missing test scenarios

---

## Running Tests

```bash
# Run all tests
cargo test

# Run specific test categories
cargo test functional_
cargo test contract::
cargo test integration::

# Run specific test files
cargo test --test functional_monitor
cargo test --test test_json_output

# Run with output
cargo test -- --nocapture
```

For more details on test execution and debugging, see the project's main documentation.