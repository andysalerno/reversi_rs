use crate::tree::Node;

use std::sync::{Arc, Weak};
use std::sync::RwLock;

#[derive(Debug)]
pub struct ArcNodeContent<T> {
    data: T,
    parent: Weak<Self>,
    children: RwLock<Vec<ArcNode<T>>>,
}

/// Wraps a NodeContent with a reference-counted owner.
pub type ArcNode<T> = Arc<ArcNodeContent<T>>;

impl<T: Clone> Node for ArcNode<T> {
    type ChildrenIter = Vec<Self>;
    type Handle = Self;
    type Data = T;

    fn data(&self) -> &T {
        &self.data
    }

    fn get_handle(&self) -> Self::Handle {
        self.clone()
    }

    fn parent(&self) -> Option<Self::Handle> {
        self.parent.upgrade().clone()
    }

    fn children(&self) -> Self::ChildrenIter {
        // TODO: perhaps this can return a borrowed slice instead,
        // if the borrow checker will allow.
        let readable_children = self.children.read().expect("Couldn't lock node children.");
        let c: Vec<Self> = readable_children.iter().cloned().collect();

        c
    }

    fn new_child(&self, data: T) -> ArcNode<T> {
        let child = Arc::new(ArcNodeContent {
            parent: Arc::downgrade(self),
            children: RwLock::default(),
            data,
        });

        let mut writable_children = self.children.write().expect("Couldn't lock node children.");
        writable_children.push(child.clone());

        child
    }

    fn new_root(data: Self::Data) -> ArcNode<T> {
        Arc::new(ArcNodeContent {
            parent: Weak::new(),
            children: RwLock::default(),
            data,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone)]
    struct DummyData;

    #[test]
    fn new_child_expects_can_add_children() {
        let root = ArcNode::new_root(DummyData);

        let root_child_a = root.new_child(DummyData);
        let root_child_a_child1 = root_child_a.new_child(DummyData);

        let root_child_b = root.new_child(DummyData);
        let root_child_b_child1 = root_child_b.new_child(DummyData);

        assert_eq!(2, root.children().into_iter().count());
        assert_eq!(1, root_child_a.children().into_iter().count());
        assert_eq!(0, root_child_a_child1.children().into_iter().count());
        assert_eq!(1, root_child_b.children().into_iter().count());
        assert_eq!(0, root_child_b_child1.children().into_iter().count());
    }

    #[test]
    fn multiple_threads_can_walk_tree() {
        let r = ArcNode::new_root(DummyData);
        let r_1 = r.new_child(DummyData);
        let r_1_1 = r_1.new_child(DummyData);
        let r_1_1_1 = r_1_1.new_child(DummyData);
        let r_1_2 = r_1.new_child(DummyData);
        let r_1_3 = r_1.new_child(DummyData);
        let r_1_3_1 = r_1_3.new_child(DummyData);

    }
}