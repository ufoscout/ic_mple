use ic_stable_structures::{log::{self, WriteError}, Memory, Storable};

use crate::log::LogStructure;

/// An extended version of the log data structure
/// that allows clearing the log.
pub struct LogExt<T: Storable, M: Memory>(Option<log::Log<T, M, M>>);

impl<T: Storable, M: Memory> LogExt<T, M> {
    /// Create new storage for values with `T` type.
    pub fn new(index_memory: M, data_memory: M) -> Self {
        // Method returns Result to be compatible with wasm implementation.
        Self(Some(log::Log::init(index_memory, data_memory)))
    }

    #[inline(always)]
    fn get_inner(&self) -> &log::Log<T, M, M> {
        self.0.as_ref().expect("inner log is always present")
    }

    #[inline(always)]
    fn mut_inner(&mut self) -> &mut log::Log<T, M, M> {
        self.0.as_mut().expect("inner log is always present")
    }
}

impl<T: Storable, M: Memory> LogStructure<T> for LogExt<T, M> {
    fn get(&self, index: u64) -> Option<T> {
        self.get_inner().get(index)
    }

    fn append(&mut self, value: T) -> Result<u64, WriteError> {
        self.mut_inner()
            .append(&value)
    }

    fn len(&self) -> u64 {
        self.get_inner().len()
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn clear(&mut self) {
        if let Some(log) = self.0.take() {
            let (index_mem, data_mem) = log.into_memories();
            self.0 = Some(log::Log::new(index_mem, data_mem));
        }
    }
}
