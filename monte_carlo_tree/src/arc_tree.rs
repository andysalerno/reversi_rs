use crate::tree::Node;

use crate::write_once_lock::{WriteOnceLock, WriteOnceWriteGuard};
use atomic_refcell::AtomicRef;
use std::sync::{Arc, Weak};

#[derive(Debug)]
pub struct ArcNodeContent<T: Send + Sync> {
    data: T,
    parent: Weak<Self>,
    children: WriteOnceLock<Vec<ArcNode<T>>>,
}

impl<T: Send + Sync> ArcNodeContent<T> {
    fn new_root_data(data: T) -> Self {
        ArcNodeContent {
            data,
            parent: Weak::new(),
            children: WriteOnceLock::new(Vec::new(), Vec::new()),
        }
    }

    fn new_child_data(parent_ptr: Weak<Self>, data: T) -> Self {
        ArcNodeContent {
            data,
            parent: parent_ptr,
            children: WriteOnceLock::new(Vec::new(), Vec::new()),
        }
    }
}

/// Wraps a NodeContent with a reference-counted owner.
pub type ArcNode<T> = Arc<ArcNodeContent<T>>;

impl<T: Send + Sync> Node for ArcNode<T> {
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

    fn children_read(&self) -> AtomicRef<Vec<Self::Handle>> {
        self.children.read()
    }

    fn children_write_lock(&self) -> WriteOnceWriteGuard<Vec<Self::Handle>> {
        self.children.write_lock()
    }

    fn new_child(&self, data: Self::Data) -> ArcNode<T> {
        let parent_ptr = Arc::downgrade(self);
        Arc::new(ArcNodeContent::new_child_data(parent_ptr, data))
    }

    fn new_root(data: Self::Data) -> ArcNode<T> {
        Arc::new(ArcNodeContent::new_root_data(data))
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

    fn add_children_to_parent(parent: &ArcNode<DummyData>, children: Vec<ArcNode<DummyData>>) {
        let parent_write_lock = parent.children_write_lock();
        parent_write_lock.write(children);
    }

    #[test]
    fn new_child_expects_can_add_children() {
        let root = ArcNode::new_root(DummyData::new());

        let root_child_a = root.new_child(DummyData::new());

        let root_child_a_child1 = root_child_a.new_child(DummyData::new());
        add_children_to_parent(&root_child_a, vec![root_child_a_child1.get_handle()]);

        let root_child_b = root.new_child(DummyData::new());
        add_children_to_parent(
            &root,
            vec![root_child_a.get_handle(), root_child_b.get_handle()],
        );

        let root_child_b_child1 = root_child_b.new_child(DummyData::new());
        add_children_to_parent(&root_child_b, vec![root_child_b_child1.get_handle()]);

        assert_eq!(2, root.children_read().iter().count());
        assert_eq!(1, root_child_a.children_read().iter().count());
        assert_eq!(0, root_child_a_child1.children_read().iter().count());
        assert_eq!(1, root_child_b.children_read().iter().count());
        assert_eq!(0, root_child_b_child1.children_read().iter().count());
    }

    #[test]
    fn multiple_threads_can_walk_tree() {
        use crossbeam::thread;

        let r = ArcNode::new_root(DummyData::new());

        let r_1 = r.new_child(DummyData::new());
        add_children_to_parent(&r, vec![r_1.get_handle()]);

        let r_1_1 = r_1.new_child(DummyData::new());
        let r_1_2 = r_1.new_child(DummyData::new());
        let r_1_3 = r_1.new_child(DummyData::new());
        add_children_to_parent(
            &r_1,
            vec![r_1_1.get_handle(), r_1_2.get_handle(), r_1_3.get_handle()],
        );

        let r_1_1_1 = r_1_1.new_child(DummyData::new());
        add_children_to_parent(&r_1_1, vec![r_1_1_1.get_handle()]);

        let r_1_3_1 = r_1_3.new_child(DummyData::new());
        add_children_to_parent(&r_1_3, vec![r_1_3_1.get_handle()]);

        thread::scope(|s| {
            for _ in 0..4 {
                s.spawn(|_| {
                    let mut node_queue = vec![r.clone()];

                    while let Some(walker) = node_queue.pop() {
                        walker.data().increment_visits();

                        let children = walker.children_read().clone();

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

    // #[test]
    // fn refcells_dont_explode() {
    //     let root = ArcNode::new_root(TestData(1));
    //     let child_1 = root.new_child(TestData(2));
    //     let child_2 = root.new_child(TestData(3));
    //     let child_3 = root.new_child(TestData(4));

    //     let child_4 = child_1.new_child(TestData(5));
    //     let child_5 = child_2.new_child(TestData(5));
    //     let child_6 = child_5.new_child(TestData(5));

    //     let child_1_children = child_1.children();
    //     let child_2_children = child_2.children();
    //     let child_3_children = child_3.children();
    //     let child_4_children = child_4.children();
    //     let child_5_children = child_5.children();
    //     let child_6_children = child_6.children();

    //     let mut _test: Vec<_> = child_6_children.iter().collect();
    //     _test = child_5_children.iter().collect();
    //     _test = child_6_children.iter().collect();
    //     _test = child_1_children.iter().collect();
    //     _test = child_2_children.iter().collect();
    //     _test = child_4_children.iter().collect();
    //     _test = child_3_children.iter().collect();
    //     _test = child_5_children.iter().collect();

    //     assert_eq!(
    //         _test[0] // child_6
    //             .parent() // child_5
    //             .unwrap()
    //             .parent() // child_2
    //             .unwrap()
    //             .parent() // root
    //             .unwrap()
    //             .data(),
    //         &TestData(1),
    //     );
    // }
}
