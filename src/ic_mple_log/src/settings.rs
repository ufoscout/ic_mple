use candid::CandidType;
use serde::Deserialize;

const DEFAULT_IN_MEMORY_RECORDS: usize = 1024;
const DEFAULT_MAX_RECORD_LENGTH: usize = 1024;

/// Log settings to initialize the logger
///
/// This structure is used to configure canisters that use `ic-log` of version `0.18` or below.
/// For newer versions of the library, use [`LogSettingsV2`] for logger configuration and
/// [`LogCanisterSettings`] for canister initialization.
#[derive(Default, Debug, Clone, CandidType, Deserialize)]
pub struct LogSettings {
    /// Enable logging to console (`ic::print` when running in IC)
    pub enable_console: bool,
    /// Number of records to be stored in the circular memory buffer.
    /// If None - storing records will be disable.
    /// If Some - should be power of two.
    pub in_memory_records: Option<usize>,
    /// Log configuration as combination of filters. By default the logger is OFF.
    /// Example of valid configurations:
    /// - info
    /// - debug,crate1::mod1=error,crate1::mod2,crate2=debug
    pub log_filter: Option<String>,
}

/// Logger settings.
///
/// For details about the fields, see docs of [`LogCanisterSettings`].
#[derive(Debug, Clone, PartialEq, Eq, CandidType, Deserialize)]
pub struct LogSettingsV2 {
    pub enable_console: bool,
    pub in_memory_records: usize,
    pub max_record_length: usize,
    pub log_filter: String,
}

impl Default for LogSettingsV2 {
    fn default() -> Self {
        Self {
            enable_console: true,
            in_memory_records: DEFAULT_IN_MEMORY_RECORDS,
            max_record_length: DEFAULT_MAX_RECORD_LENGTH,
            log_filter: "debug".to_string(),
        }
    }
}

