use crate::models::{MonitoredProcess, ProcessSnapshot};

/// Manages process state tracking between polling cycles
pub struct ProcessTracker {
    current_snapshot: Option<ProcessSnapshot>,
}

impl ProcessTracker {
    pub fn new() -> Self {
        Self {
            current_snapshot: None,
        }
    }

    /// Detect new processes by comparing current snapshot with previous
    pub fn detect_new_processes(
        &mut self,
        new_snapshot: ProcessSnapshot,
    ) -> Vec<MonitoredProcess> {
        let new_processes = match &self.current_snapshot {
            None => {
                // First snapshot - all processes are "new" but we ignore them
                // to avoid flooding output on startup
                Vec::new()
            }
            Some(previous) => new_snapshot.new_processes(previous),
        };

        self.current_snapshot = Some(new_snapshot);
        new_processes
    }

    /// Apply path filters to processes (reusing existing scan logic)
    pub fn apply_path_filters(
        processes: Vec<MonitoredProcess>,
        path_filters: &[std::path::PathBuf],
    ) -> Vec<MonitoredProcess> {
        if path_filters.is_empty() {
            return processes;
        }

        processes
            .into_iter()
            .filter(|process| {
                path_filters.iter().any(|filter_path| {
                    process.executable_path.starts_with(filter_path)
                })
            })
            .collect()
    }

    /// Apply entitlement filters to processes using consistent pattern matching
    pub fn apply_entitlement_filters(
        processes: Vec<MonitoredProcess>,
        entitlement_filters: &[String],
    ) -> Vec<MonitoredProcess> {
        use crate::entitlements::pattern_matcher;
        
        processes
            .into_iter()
            .filter(|process| {
                pattern_matcher::entitlements_match_filters(&process.entitlements, entitlement_filters)
            })
            .collect()
    }
}