use std::borrow::Cow;

use ic_stable_structures::{Memory, StableCell, Storable};

pub mod versioned;

pub trait CellStructure<T: Clone> {
    /// Returns the current value in the cell.
    fn get(&self) -> Cow<'_, T>;

    /// Updates the current value in the cell.
    fn set(&mut self, value: T);
}

impl<T: Storable + Clone, M: Memory> CellStructure<T> for StableCell<T, M> {
    fn get(&self) -> Cow<'_, T> {
        Cow::Borrowed(self.get())
    }

    fn set(&mut self, value: T) {
        self.set(value);
    }
}
