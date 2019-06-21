/// This is a simple, generic reference-counted implementation of the Node trait.
use crate::Node;
use std::cell::RefCell;
use std::rc::{Rc, Weak};

#[derive(Debug)]
pub struct NodeContent<T> {
    data: T,
    parent: Weak<Self>,
    children: RefCell<Vec<RcNode<T>>>,
}

/// Wraps a NodeContent with a reference-counted owner.
pub type RcNode<T> = Rc<NodeContent<T>>;

impl<T: Clone> Node for RcNode<T> {
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
        let c: Vec<Self> = self.children.borrow().iter().cloned().collect();

        c
    }

    fn new_child(&self, data: T) -> RcNode<T> {
        let child = Rc::new(NodeContent {
            parent: Rc::downgrade(self),
            children: RefCell::default(),
            data,
        });

        self.children.borrow_mut().push(child.clone());

        child
    }

    fn new_root(data: Self::Data) -> RcNode<T> {
        Rc::new(NodeContent {
            parent: Weak::new(),
            children: RefCell::default(),
            data,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Copy, Clone, PartialEq, Debug)]
    struct TestData(i32);

    #[test]
    fn node_holds_data() {
        let root = RcNode::new_root(TestData(1));

        assert_eq!(root.data(), &TestData(1));
    }

    #[test]
    fn node_adds_child() {
        let root = RcNode::new_root(TestData(1));
        let child = root.new_child(TestData(2));

        let root_children = root.children();

        assert_eq!(root_children[0].data(), &TestData(2));
        assert_eq!(child.children().len(), 0);
    }

    #[test]
    fn child_has_parent() {
        let root = RcNode::new_root(TestData(1));
        let child = root.new_child(TestData(2));

        assert_eq!(
            child.parent().expect("child should have a parent").data(),
            &TestData(1)
        );
    }

    #[test]
    fn root_has_no_parent() {
        let root = RcNode::new_root(TestData(1));

        assert!(root.parent().is_none());
    }

    #[test]
    fn refcells_dont_explode() {
        let root = RcNode::new_root(TestData(1));
        let child_1 = root.new_child(TestData(2));
        let child_2 = root.new_child(TestData(3));
        let child_3 = root.new_child(TestData(4));

        let child_4 = child_1.new_child(TestData(5));
        let child_5 = child_2.new_child(TestData(5));
        let child_6 = child_5.new_child(TestData(5));

        let child_1_children = child_1.children();
        let child_2_children = child_2.children();
        let child_3_children = child_3.children();
        let child_4_children = child_4.children();
        let child_5_children = child_5.children();
        let child_6_children = child_6.children();

        let mut _test: Vec<_> = child_6_children.iter().collect();
        _test = child_5_children.iter().collect();
        _test = child_6_children.iter().collect();
        _test = child_1_children.iter().collect();
        _test = child_2_children.iter().collect();
        _test = child_4_children.iter().collect();
        _test = child_3_children.iter().collect();
        _test = child_5_children.iter().collect();

        assert_eq!(
            _test[0] // child_6
                .parent() // child_5
                .unwrap()
                .parent() // child_2
                .unwrap()
                .parent() // root
                .unwrap()
                .data(),
            &TestData(1),
        );
    }
}
