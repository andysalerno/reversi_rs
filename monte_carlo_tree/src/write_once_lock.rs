use atomic_refcell::{AtomicRef, AtomicRefCell};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, MutexGuard};

/// A lock that assumes writing will only
/// happen once, and all read attemps
/// will occur after the write has completed.
/// The goal is similar to RwLock, but without
/// the cost of acquiring a read lock on every read
/// (since by that point we know locking is no longer required
/// as writing is over).
#[derive(Debug)]
pub(crate) struct WriteOnceLock<T> {
    data_write: Mutex<()>,
    data_read: AtomicRefCell<T>,
    has_written: AtomicBool,
}

impl<T: Sized> WriteOnceLock<T> {
    pub fn new(data: T) -> Self {
        Self {
            data_write: Mutex::new(()),
            data_read: AtomicRefCell::new(data),
            has_written: AtomicBool::new(false),
        }
    }

    /// If this is sequentially first invocation of this call on any thread,
    /// acquires a lock, otherwise returns None.
    /// The holder of this lock can safely call `write()`.
    pub fn write_lock(&self) -> MutexGuard<()> {
        self.data_write.lock().expect("Acquiring data write lock.")
    }

    /// Writes data to this wrapper's interior data store.
    /// The expectation is: you only call this while you have a valid MutextGuard
    /// from `write_lock()` in scope (NOT enforced by code!)
    pub fn write(&self, data: T) {
        *self.data_read.borrow_mut() = data;
    }

    /// Reads the data that was previously written into this wrapper's data store.
    /// Panics if the data store was not previously written to.
    pub fn read(&self) -> AtomicRef<T> {
        let has_written = self.has_written.load(Ordering::SeqCst);
        assert!(
            has_written,
            "Attempt to read from a WriteOnceLock before writing."
        );

        self.data_read.borrow()
    }
}
