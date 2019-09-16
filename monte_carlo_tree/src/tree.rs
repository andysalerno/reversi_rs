use std::borrow::Borrow;
use atomic_refcell::AtomicRef;
use std::sync::MutexGuard;

/// A tree node that can hold data, and refer to
/// its parent and children.
pub trait Node: Sized {
    type Handle: Borrow<Self> + Clone;
    type Data;

    fn data(&self) -> &Self::Data;
    fn parent(&self) -> Option<Self::Handle>;
    fn get_handle(&self) -> Self::Handle;

    fn children_write_lock(&self) -> Option<MutexGuard<()>>;
    fn children_write(&self, data: Vec<Self::Handle>);
    fn children_read(&self) -> AtomicRef<Vec<Self::Handle>>;

    fn new_root(state: Self::Data) -> Self::Handle;
    fn new_child(&self, state: Self::Data) -> Self::Handle;
}
