use candid::CandidType;
use env_filter::ParseError;
use log::SetLoggerError;
use serde::Deserialize;

/// Specifies what to take from a long list of items.
#[derive(Debug, Copy, Clone, CandidType, Deserialize)]
pub struct Pagination {
    /// First item id to get.
    pub offset: usize,
    /// Max number of items to get.
    pub count: usize,
}

/// Error returned by the logger canister.
#[derive(Debug, Clone, CandidType, Deserialize, Eq, PartialEq)]
pub enum LogError {
    /// An initialization was called for the logger, but it is already initialized.
    AlreadyInitialized,
    /// The logger is not initialized.
    NotInitialized,
    /// The caller does not have permission to execute this method.
    NotAuthorized,
    /// Something bad happened.
    Generic(String),
    /// The given memory cannot be used to store logger configuration.
    InvalidMemory,
    /// Error in the logger configuration.
    InvalidConfiguration(String),
}

impl From<ParseError> for LogError {
    fn from(value: ParseError) -> Self {
        Self::InvalidConfiguration(value.to_string())
    }
}

impl From<SetLoggerError> for LogError {
    fn from(_: SetLoggerError) -> Self {
        Self::AlreadyInitialized
    }
}
