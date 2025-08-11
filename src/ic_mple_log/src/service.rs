use std::borrow::Cow;

use crate::types::LogError;
use crate::{LogSettings, LoggerConfigHandle, init_log};
use candid::{CandidType, Decode, Encode};
pub use ic_mple_utils::store::Storage;
use ic_stable_structures::DefaultMemoryImpl;
use ic_stable_structures::memory_manager::VirtualMemory;
use ic_stable_structures::storable::Bound;
use ic_stable_structures::{StableCell, Storable};
use serde::Deserialize;

impl Storable for LogSettings {
    const BOUND: Bound = Bound::Unbounded;

    fn to_bytes(&self) -> std::borrow::Cow<'_, [u8]> {
        Cow::from(Encode!(&self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(&bytes, LogSettings).unwrap()
    }

    fn into_bytes(self) -> Vec<u8> {
        Encode!(&self).unwrap()
    }
}

const DEFAULT_IN_MEMORY_RECORDS: usize = 1024;
const DEFAULT_MAX_RECORD_LENGTH: usize = 1024;

/// Log settings to initialize the logger
#[derive(Default, Debug, Clone, CandidType, Deserialize, PartialEq, Eq)]
pub struct LogServiceSettings {
    /// Enable logging to console (`ic::print` when running in IC).
    /// If `None`, default value will be used (`false`).
    pub enable_console: Option<bool>,

    /// Number of records to be stored in the circular memory buffer.
    ///
    /// If set to 0, logging will be disabled.
    ///
    /// If `None`, default value will be used (`1024`).
    pub in_memory_records: Option<usize>,

    /// Maximum length (in bytes) of a single log entry.
    ///
    /// If set to 0, the log will still add entries to the log, but they all will contain only an
    /// empty string.
    ///
    /// If `None`, default value will be used (`1024`).
    pub max_record_length: Option<usize>,

    /// Log configuration as combination of filters. By default, the logger filter is set to `warn`.
    ///
    /// Example of valid configurations:
    /// - info
    /// - debug,crate1::mod1=error,crate1::mod2,crate2=debug
    pub log_filter: Option<String>,
}

impl From<LogServiceSettings> for LogSettings {
    fn from(settings: LogServiceSettings) -> Self {
        Self {
            enable_console: settings.enable_console.unwrap_or(false),
            in_memory_records: settings
                .in_memory_records
                .unwrap_or(DEFAULT_IN_MEMORY_RECORDS),
            max_record_length: settings
                .max_record_length
                .unwrap_or(DEFAULT_MAX_RECORD_LENGTH),
            log_filter: settings.log_filter.unwrap_or("warn".to_string()),
        }
    }
}

pub type LoggerServiceStorage = StableCell<LogSettings, VirtualMemory<DefaultMemoryImpl>>;

/// Handles the runtime logger configuration
pub struct LoggerConfigService<
    S: Storage<LoggerServiceStorage>,
> {
    pub logger_config: Option<LoggerConfigHandle>,
    pub log_settings_store: S,
}

impl<S: Storage<LoggerServiceStorage>> LoggerConfigService<S> {
    /// Instantiates a new LoggerConfigService
    pub fn new(log_settings_store: S) -> Self {
        Self {
            logger_config: None,
            log_settings_store,
        }
    }

    /// Initialize logger. Must be called just once in the canister init and post_upgrade hook
    pub fn init(&mut self, log_settings: Option<LogServiceSettings>) -> Result<(), LogError> {
        if self.logger_config.is_some() {
            return Err(LogError::AlreadyInitialized);
        }

        if let Some(log_settings) = log_settings {
            self.log_settings_store.with_borrow_mut(|store| {
                store.set(log_settings.into());
            });
        }

        self.log_settings_store.with_borrow(|store| {
            self.logger_config = Some(init_log(store.get())?);
            Ok(())
        })
    }

    /// Changes the logger filter at runtime
    pub fn set_logger_filter(&mut self, filter: &str) -> Result<(), LogError> {
        self.update_log_settings(filter)?;
        match self.logger_config.as_mut() {
            Some(logger_config) => logger_config.update_filters(filter),
            None => Err(LogError::NotInitialized),
        }
    }

    /// Returns the current logger filter
    pub fn get_logger_filter(&self) -> String {
        self.log_settings_store
            .with_borrow(|store| store.get().log_filter.clone())
    }

    fn update_log_settings(&mut self, filter: &str) -> Result<(), LogError> {
        self.log_settings_store.with_borrow_mut(|store| {
            let mut log_settings = store.get().clone();
            log_settings.log_filter = filter.to_string();
            store.set(log_settings);
        });

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::cell::RefCell;

    use ic_stable_structures::{
        DefaultMemoryImpl,
        memory_manager::{MemoryId, MemoryManager},
    };

    use super::*;

    thread_local! {
        static LOG_SETTINGS_STORE: RefCell<LoggerServiceStorage> = RefCell::new(
            StableCell::new(MemoryManager::init(DefaultMemoryImpl::default()).get(MemoryId::new(1)), LogSettings::default())
        );
    }

    #[test]
    fn test_logger_config_service_with_thread_local() {
        let logger_config_service = LoggerConfigService::new(&LOG_SETTINGS_STORE);
        assert!(logger_config_service.logger_config.is_none());
        assert_eq!(logger_config_service.get_logger_filter(), "warn");
    }

    #[test]
    fn test_logger_config_service_with_local_var() {
        let store = StableCell::new(
            MemoryManager::init(DefaultMemoryImpl::default()).get(MemoryId::new(1)),
            LogSettings::default(),
        );
        let logger_config_service = LoggerConfigService::new(store);
        assert!(logger_config_service.logger_config.is_none());
        assert_eq!(logger_config_service.get_logger_filter(), "warn");
    }
}
