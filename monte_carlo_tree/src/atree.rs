use crate::tree::Node;
use std::marker::Sync;

/// Describes a Node that is also Sync.
pub trait ANode: Node + Sync {}
