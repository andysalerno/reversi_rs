use crate::tree::Node;
use crate::atree::ANode;

use std::sync::RwLock;
use std::sync::{Arc, Weak};

#[derive(Debug)]
pub struct ArcNodeContent<T> {
    data: T,
    parent: Weak<Self>,
    children: RwLock<Vec<ArcNode<T>>>,
}

/// Wraps a NodeContent with a reference-counted owner.
pub type ArcNode<T> = Arc<ArcNodeContent<T>>;

impl<T> Node for ArcNode<T> {
    type ChildrenIter = Vec<Self>;
    type Handle = Self;
    type Data = T;

    fn data(&self) -> &Self::Data {
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
        let readable_children = self.children.read().expect("Couldn't lock node children for reading.");
        let c: Vec<Self> = readable_children.iter().cloned().collect();

        c
    }

    fn new_child(&self, data: T) -> ArcNode<T> {
        let child = Arc::new(ArcNodeContent {
            parent: Arc::downgrade(self),
            children: RwLock::default(),
            data,
        });

        let mut writable_children = self.children.write().expect("Couldn't lock node children for writing.");
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

impl<T: std::marker::Send + std::marker::Sync> ANode for ArcNode<T> {
    fn children_write_lock(&self) -> std::sync::RwLockWriteGuard<Self::ChildrenIter> {
        self.children.write().expect("Couldn't lock children for writing.")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct DummyData {
        visits: AtomicUsize,
    }

    impl DummyData {
        fn new() -> Self {
            DummyData {
                visits: AtomicUsize::new(0),
            }
        }

        fn increment_visits(&self) {
            self.visits.fetch_add(1, Ordering::Relaxed);
        }

        fn get_visits(&self) -> usize {
            self.visits.load(Ordering::SeqCst)
        }
    }

    #[test]
    fn new_child_expects_can_add_children() {
        let root = ArcNode::new_root(DummyData::new());

        let root_child_a = root.new_child(DummyData::new());
        let root_child_a_child1 = root_child_a.new_child(DummyData::new());

        let root_child_b = root.new_child(DummyData::new());
        let root_child_b_child1 = root_child_b.new_child(DummyData::new());

        assert_eq!(2, root.children().into_iter().count());
        assert_eq!(1, root_child_a.children().into_iter().count());
        assert_eq!(0, root_child_a_child1.children().into_iter().count());
        assert_eq!(1, root_child_b.children().into_iter().count());
        assert_eq!(0, root_child_b_child1.children().into_iter().count());
    }

    #[test]
    fn multiple_threads_can_walk_tree() {
        use crossbeam::thread;

        let r = ArcNode::new_root(DummyData::new());
        let r_1 = r.new_child(DummyData::new());
        let r_1_1 = r_1.new_child(DummyData::new());
        let r_1_1_1 = r_1_1.new_child(DummyData::new());
        let r_1_2 = r_1.new_child(DummyData::new());
        let r_1_3 = r_1.new_child(DummyData::new());
        let r_1_3_1 = r_1_3.new_child(DummyData::new());

        thread::scope(|s| {
            for _ in 0..4 {
                s.spawn(|_| {
                    let mut node_queue = vec![r.clone()];

                    while let Some(walker) = node_queue.pop() {
                        walker.data().increment_visits();

                        let children = walker.children();
                        let children = children.into_iter().collect::<Vec<_>>();

                        node_queue.extend(children);
                    }
                });
            }
        })
        .expect("Scope didn't terminate properly.");

        assert_eq!(4, r.data().get_visits());
        assert_eq!(4, r_1.data().get_visits());
        assert_eq!(4, r_1_1.data().get_visits());
        assert_eq!(4, r_1_1_1.data().get_visits());
        assert_eq!(4, r_1_2.data().get_visits());
        assert_eq!(4, r_1_3.data().get_visits());
        assert_eq!(4, r_1_3_1.data().get_visits());
    }
}
