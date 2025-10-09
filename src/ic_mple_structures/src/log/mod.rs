use ic_stable_structures::log::WriteError;

pub mod ext;

pub trait LogStructure<T> {
    /// Returns reference to value stored in stable memory.
    fn get(&self, index: u64) -> Option<T>;

    /// Updates value in stable memory.
    fn append(&mut self, value: T) -> Result<u64, WriteError>;

    /// Number of values in the log.
    fn len(&self) -> u64;

    // Returns true, if the Log doesn't contain any values.
    fn is_empty(&self) -> bool;

    /// Remove all items from the log.
    fn clear(&mut self);
}

