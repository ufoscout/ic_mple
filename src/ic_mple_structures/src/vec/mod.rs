pub mod vec;

pub trait VecStructure<T> {
    /// Returns if vector is empty
    fn is_empty(&self) -> bool;

    /// Removes al the values from the vector
    fn clear(&mut self);

    /// Returns the number of elements in the vector
    fn len(&self) -> u64;

    /// Sets the value at `index` to `item`
    /// WARN: this panics if index out of range
    fn set(&mut self, index: u64, item: &T);

    /// Returns the value at `index`
    fn get(&self, index: u64) -> Option<T>;

    /// Appends new value to the vector
    fn push(&mut self, item: &T);

    /// Pops the last value from the vector
    fn pop(&mut self) -> Option<T>;
}
