use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

pub(crate) fn clone_atomic_usize(atom: &AtomicUsize) -> AtomicUsize {
    let raw = atom.load(Ordering::SeqCst);
    AtomicUsize::new(raw)
}

pub(crate) fn clone_atomic_bool(atom: &AtomicBool) -> AtomicBool {
    let raw = atom.load(Ordering::SeqCst);
    AtomicBool::new(raw)
}
