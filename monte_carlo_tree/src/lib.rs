pub mod rc_tree;

use std::borrow::Borrow;

pub trait Node
where
    Self: Sized,
{
    type ChildrenIter: IntoIterator<Item = Self>;
    type ParentBorrow: Borrow<Self>;
    type Data;

    fn data(&self) -> &Self::Data;
    fn parent(&self) -> Option<Self::ParentBorrow>;
    fn children(&self) -> Self::ChildrenIter;

    fn new_child(&self, state: &Self::Data) -> Self;
    fn new_root(state: Self::Data) -> Self;
}

#[cfg(test)]
mod test {

    use super::*;

    struct TestNode;

    impl Node for TestNode {
        type ChildrenIter = Vec<Self>;
        type ParentBorrow = Self;
        type Data = Option<()>;

        fn data(&self) -> &Self::Data {
            unimplemented!()
        }
        fn parent(&self) -> Option<Self::ParentBorrow> {
            unimplemented!()
        }
        fn children(&self) -> Self::ChildrenIter {
            Vec::new()
        }
        fn new_child(&self, state: &Self::Data) -> Self {
            unimplemented!()
        }
        fn new_root(state: Self::Data) -> Self {
            unimplemented!()
        }
    }

    fn stub() -> impl Node {
        TestNode
    }

    #[test]
    fn can_use_it() {
        let test_node = stub();
        let children = test_node.children();

        for _child in children {}
    }

}
