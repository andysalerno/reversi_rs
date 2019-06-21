/// This describes the general Node trait that can be used for making trees (specifically, monte-carlo trees)
pub mod rc_tree;

use std::borrow::Borrow;

pub trait Node: Sized + Clone {
    type ChildrenIter: IntoIterator<Item = Self::Handle>;
    type Handle: Borrow<Self>;
    type Data;

    fn data(&self) -> &Self::Data;
    fn parent(&self) -> Option<Self::Handle>;
    fn get_handle(&self) -> Self::Handle;
    fn children(&self) -> Self::ChildrenIter;

    fn new_child(&self, state: Self::Data) -> Self::Handle;
    fn new_root(state: Self::Data) -> Self::Handle;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone)]
    struct TestNode;

    impl Node for TestNode {
        type ChildrenIter = Vec<Self>;
        type Handle = Self;
        type Data = Option<()>;

        fn data(&self) -> &Self::Data {
            &None
        }
        fn parent(&self) -> Option<Self::Handle> {
            unimplemented!()
        }
        fn get_handle(&self) -> Self::Handle {
            unimplemented!()
        }
        fn children(&self) -> Self::ChildrenIter {
            Vec::new()
        }
        fn new_child(&self, _state: Self::Data) -> Self {
            unimplemented!()
        }
        fn new_root(_state: Self::Data) -> Self {
            unimplemented!()
        }
    }

    fn stub() -> impl Node {
        TestNode
    }

    #[test]
    fn can_iter_over_generic_children() {
        let test_node = stub();
        let children = test_node.children();

        for _child in children {}
    }

}
