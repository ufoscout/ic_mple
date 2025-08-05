/*
Timestamps aren't available when we don't have a `humantime` dependency.
*/
use crate::{fmt::Formatter, platform};

pub(in crate::fmt) mod glob {}

impl Formatter {
    /// Get a [`Timestamp`] for the current date and time in UTC with
    /// nanosecond precision.
    pub fn timestamp_nanos(&self) -> u64 {
        platform::current_timestamp_in_nanosecs()
    }
}
