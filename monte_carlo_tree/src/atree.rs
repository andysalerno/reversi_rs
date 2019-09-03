use crate::tree::Node;
use std::marker::Sync;
use std::sync::RwLock;

pub trait ANode: Node + Sync {
    fn children_locked(&self) -> &RwLock<Vec<Self>>;
}