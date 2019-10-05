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
    default_value: AtomicRefCell<T>,
    data_read: AtomicRefCell<T>,
    has_written: AtomicBool,
}

pub struct WriteOnceWriteGuard<'a, T> {
    _mutex_guard: MutexGuard<'a, ()>,
    guarded: &'a AtomicRefCell<T>,
    has_written: &'a AtomicBool,
}

impl<'a, T: 'a> WriteOnceWriteGuard<'a, T> {
    pub fn new(
        mutex_guard: MutexGuard<'a, ()>,
        guarded: &'a AtomicRefCell<T>,
        has_written: &'a AtomicBool,
    ) -> Self {
        Self {
            _mutex_guard: mutex_guard,
            guarded,
            has_written,
        }
    }

    pub fn write(&self, data: T) {
        *self.guarded.borrow_mut() = data;
        self.has_written.store(true, Ordering::SeqCst);
    }
}

impl<T: Sized> WriteOnceLock<T> {
    pub fn new(data: T, default_data: T) -> Self {
        Self {
            data_write: Mutex::new(()),
            data_read: AtomicRefCell::new(data),
            default_value: AtomicRefCell::new(default_data),
            has_written: AtomicBool::new(false),
        }
    }

    pub fn write_lock(&self) -> WriteOnceWriteGuard<T> {
        let write_lock = self.data_write.lock().expect("Acquiring data write lock.");
        WriteOnceWriteGuard::new(write_lock, &self.data_read, &self.has_written)
    }

    /// Reads the data that was previously written into this wrapper's data store.
    /// Panics if the data store was not previously written to.
    pub fn read(&self) -> AtomicRef<T> {
        let has_written = self.has_written.load(Ordering::SeqCst);

        if has_written {
            self.data_read.borrow()
        } else {
            self.default_value.borrow()
        }
    }
}
