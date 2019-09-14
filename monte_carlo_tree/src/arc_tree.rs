use crate::atree::ANode;
use crate::tree::Node;

use std::sync::{Arc, Weak};
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

#[derive(Debug)]
pub struct ArcNodeContent<T> {
    data: T,
    parent: Weak<Self>,
    children: RwLock<Vec<ArcNode<T>>>,
}

/// Wraps a NodeContent with a reference-counted owner.
pub type ArcNode<T> = Arc<ArcNodeContent<T>>;

impl<T> Node for ArcNode<T> {
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

    fn children_lock_read(&self) -> RwLockReadGuard<Vec<Self::Handle>> {
        self.children
            .read()
            .expect("Couldn't acquire children read lock")
    }

    fn children_lock_write(&self) -> RwLockWriteGuard<Vec<Self::Handle>> {
        self.children
            .write()
            .expect("Couldn't acquire children write lock")
    }

    fn children_handles(&self) -> Vec<Self::Handle> {
        let children_read = self.children_lock_read();

        children_read.iter().cloned().collect()
    }

    fn new_child(
        &self,
        data: T,
        write_lock: &mut RwLockWriteGuard<Vec<Self::Handle>>,
    ) -> ArcNode<T> {
        let child = Arc::new(ArcNodeContent {
            parent: Arc::downgrade(self),
            children: RwLock::default(),
            data,
        });

        write_lock.push(child.clone());

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

impl<T: std::marker::Send + std::marker::Sync> ANode for ArcNode<T> {}

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

        let root_child_a = root.new_child(DummyData::new(), &mut root.children_lock_write());
        let root_child_a_child1 =
            root_child_a.new_child(DummyData::new(), &mut root_child_a.children_lock_write());

        let root_child_b = root.new_child(DummyData::new(), &mut root.children_lock_write());
        let root_child_b_child1 =
            root_child_b.new_child(DummyData::new(), &mut root_child_b.children_lock_write());

        assert_eq!(2, root.children_handles().into_iter().count());
        assert_eq!(1, root_child_a.children_handles().into_iter().count());
        assert_eq!(
            0,
            root_child_a_child1.children_handles().into_iter().count()
        );
        assert_eq!(1, root_child_b.children_handles().into_iter().count());
        assert_eq!(
            0,
            root_child_b_child1.children_handles().into_iter().count()
        );
    }

    #[test]
    fn multiple_threads_can_walk_tree() {
        use crossbeam::thread;

        let r = ArcNode::new_root(DummyData::new());
        let r_1 = r.new_child(DummyData::new(), &mut r.children_lock_write());
        let r_1_1 = r_1.new_child(DummyData::new(), &mut r_1.children_lock_write());
        let r_1_1_1 = r_1_1.new_child(DummyData::new(), &mut r_1_1.children_lock_write());
        let r_1_2 = r_1.new_child(DummyData::new(), &mut r_1.children_lock_write());
        let r_1_3 = r_1.new_child(DummyData::new(), &mut r_1.children_lock_write());
        let r_1_3_1 = r_1_3.new_child(DummyData::new(), &mut r_1_3.children_lock_write());

        thread::scope(|s| {
            for _ in 0..4 {
                s.spawn(|_| {
                    let mut node_queue = vec![r.clone()];

                    while let Some(walker) = node_queue.pop() {
                        walker.data().increment_visits();

                        let children = walker.children_handles().clone();

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
