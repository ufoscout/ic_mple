use std::{cell::RefCell, thread::LocalKey};

/// An abstract storage interface that allows creating services that can
/// use both thread-local and owned plain object storage.
/// This simplifies unit testing.
pub trait Storage<T> {
    /// Acquires a mutable reference to the contained value.
    fn with_borrow_mut<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut T) -> R;

    /// Acquires a reference to the contained value.
    fn with_borrow<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&T) -> R;
}

impl<T: 'static> Storage<T> for &'static LocalKey<RefCell<T>> {
    fn with_borrow_mut<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        LocalKey::with_borrow_mut(self, f)
    }

    fn with_borrow<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        LocalKey::with_borrow(self, f)
    }
}

impl<T> Storage<T> for T {
    fn with_borrow_mut<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        f(self)
    }

    fn with_borrow<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        f(self)
    }
}
