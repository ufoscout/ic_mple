use candid::CandidType;
use serde::Deserialize;

const DEFAULT_IN_MEMORY_RECORDS: usize = 1024;
const DEFAULT_MAX_RECORD_LENGTH: usize = 1024;

/// Logger settings.
#[derive(Debug, Clone, PartialEq, Eq, CandidType, Deserialize)]
pub struct LogSettings {
    /// Enable logging to console (`ic::print` when running in IC)
    pub enable_console: bool,
    /// Number of records to be stored in the logger in memory queue.
    /// Default value is 1024.
    pub in_memory_records: usize,
    /// Maximum length (in bytes) of a single log entry in the logger in memory queue.
    /// Default value is 1024.
    pub max_record_length: usize,
    /// Log configuration as combination of filters.
    /// Example of valid configurations:
    /// - info
    /// - debug,crate1::mod1=error,crate1::mod2,crate2=debug
    pub log_filter: String,
}

impl Default for LogSettings {
    fn default() -> Self {
        Self {
            enable_console: false,
            in_memory_records: DEFAULT_IN_MEMORY_RECORDS,
            max_record_length: DEFAULT_MAX_RECORD_LENGTH,
            log_filter: "warn".to_string(),
        }
    }
}

