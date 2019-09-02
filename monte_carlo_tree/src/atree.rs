use crate::tree::Node;
use std::marker::Sync;

pub trait ANode: Node + Sync {
    fn children_write_lock(&self) -> std::sync::RwLockWriteGuard<Self::ChildrenIter>;
}