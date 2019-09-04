use std::borrow::Borrow;

/// A tree node that can hold data, and refer to
/// its parent and children.
pub trait Node: Sized {
    type Handle: Borrow<Self> + Clone;
    type Data;

    fn data(&self) -> &Self::Data;
    fn parent(&self) -> Option<Self::Handle>;
    fn get_handle(&self) -> Self::Handle;
    fn children(&self) -> &Vec<Self::Handle>;

    fn new_child(&self, state: Self::Data) -> Self::Handle;
    fn new_root(state: Self::Data) -> Self::Handle;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone)]
    struct TestNode {
        children: Vec<Self>
    }

    impl Node for TestNode {
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
        fn children(&self) -> &Vec<Self> {
            &self.children
        }
        fn new_child(&self, _state: Self::Data) -> Self {
            unimplemented!()
        }
        fn new_root(_state: Self::Data) -> Self {
            unimplemented!()
        }
    }

    fn stub() -> impl Node {
        TestNode {
            children: Vec::new()
        }
    }

    #[test]
    fn can_iter_over_generic_children() {
        let test_node = stub();
        let children = test_node.children();

        for _child in children {}
    }

}
