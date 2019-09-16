use std::sync::{Mutex, MutexGuard};
use std::sync::atomic::{AtomicBool, Ordering};
use atomic_refcell::{AtomicRefCell, AtomicRef};
use std::ops::Deref;

/// A lock that assumes writing will only
/// happen once, and all read attemps
/// will occur after the write has completed.
/// The goal is similar to RwLock, but without
/// the cost of acquiring a read lock on every read
/// (since by that point we know locking is no longer required
/// as writing is over).
#[derive(Debug)]
pub(crate) struct WriteOnceLock<T> {
    data_write: Mutex<T>,
    data_read: AtomicRefCell<T>,
    has_written: AtomicBool, 
}

impl<T: Clone + Sized> WriteOnceLock<T> {
    pub fn new(data: T) -> Self {
        Self {
            data_write: Mutex::new(data.clone()),
            data_read: AtomicRefCell::new(data),
            has_written: AtomicBool::new(false), 
        }
    }

    pub fn write_lock(&self) -> MutexGuard<T> {
        let has_written = self.has_written.swap(true, Ordering::SeqCst);
        assert!(!has_written, "Attempt to right twice on a WriteOnceLock");

        let write_lock = self.data_write.lock().expect("Acquiring data write lock.");

        // *self.data_read.borrow_mut() = data;

        write_lock
    }

    pub fn read(&self) -> AtomicRef<T> {
        let has_written = self.has_written.load(Ordering::SeqCst);
        assert!(has_written, "Attempt to read from a WriteOnceLock before writing.");

        self.data_read.borrow()
    }
}