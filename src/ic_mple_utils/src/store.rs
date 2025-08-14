use std::{cell::RefCell, rc::Rc, sync::{Arc, Mutex, RwLock}, thread::LocalKey};

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

impl <T> Storage<T> for RefCell<T> {

    fn with_borrow_mut<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        f(&mut self.borrow_mut())
    }

    fn with_borrow<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        f(&self.borrow())
    }
}

impl <T> Storage<T> for Rc<RefCell<T>> {

    fn with_borrow_mut<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        f(&mut self.borrow_mut())
    }

    fn with_borrow<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        f(&self.borrow())
    }
}

impl <T> Storage<T> for Mutex<T> {

    fn with_borrow_mut<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        f(&mut self.get_mut().unwrap())
    }

    fn with_borrow<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        f(&self.lock().unwrap())
    }
}

impl <T> Storage<T> for Arc<Mutex<T>> {

    fn with_borrow_mut<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        f(&mut self.lock().unwrap())
    }

    fn with_borrow<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        f(&self.lock().unwrap())
    }
}

impl <T> Storage<T> for RwLock<T> {

    fn with_borrow_mut<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        f(&mut self.write().unwrap())
    }

    fn with_borrow<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        f(&self.read().unwrap())
    }
}

impl <T> Storage<T> for Arc<RwLock<T>> {

    fn with_borrow_mut<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        f(&mut self.write().unwrap())
    }

    fn with_borrow<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        f(&self.read().unwrap())
    }
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