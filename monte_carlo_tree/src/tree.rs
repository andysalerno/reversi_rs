use std::borrow::Borrow;
use std::sync::{RwLockReadGuard, RwLockWriteGuard};

/// A tree node that can hold data, and refer to
/// its parent and children.
pub trait Node: Sized {
    type Handle: Borrow<Self> + Clone;
    type Data;

    fn data(&self) -> &Self::Data;
    fn parent(&self) -> Option<Self::Handle>;
    fn get_handle(&self) -> Self::Handle;

    fn children_lock_read(&self) -> RwLockReadGuard<Vec<Self::Handle>>;
    fn children_lock_write(&self) -> RwLockWriteGuard<Vec<Self::Handle>>;
    fn children_handles(&self) -> Vec<Self::Handle>;

    fn new_child(&self, state: Self::Data, write_lock: &mut RwLockWriteGuard<Vec<Self::Handle>>) -> Self::Handle;
    fn new_root(state: Self::Data) -> Self::Handle;
}