/// This describes the general Node trait that can be used for making trees (specifically, monte-carlo trees)
pub mod rc_tree;

use std::borrow::Borrow;

pub trait Node: Sized + Clone {
    type ChildrenIter: IntoIterator<Item = Self>;
    type Borrowable: Borrow<Self>;
    type Data;

    fn data(&self) -> &Self::Data;
    fn parent(&self) -> Option<Self::Borrowable>;
    fn make_borrowable(&self) -> Self::Borrowable;
    fn children(&self) -> Self::ChildrenIter;

    fn new_child(&self, state: Self::Data) -> Self;
    fn new_root(state: Self::Data) -> Self;
}

#[cfg(test)]
mod tests {

    use super::*;

    #[derive(Copy, Clone, Default)]
    struct ArenaIndex(usize);

    #[derive(Default)]
    struct ArenaData {
        index: ArenaIndex,
        parent_idx: Option<ArenaIndex>,
    }

    struct Arena<T: Node> {
        arena: Vec<(T, ArenaData)>,
    }

    impl<T: Node> Arena<T> {
        fn get_node(&self, index: ArenaIndex) -> &T {
            &self.arena[index.0].0
        }

        fn get_node_mut(&mut self, index: ArenaIndex) -> &mut T {
            &mut self.arena[index.0].0
        }

        fn insert_root(&mut self, data: T) -> ArenaIndex {
            let index = ArenaIndex(self.arena.len());

            self.arena.push((
                data,
                ArenaData {
                    index,
                    ..Default::default()
                },
            ));

            index
        }

        fn insert_node(&mut self, parent_idx: ArenaIndex, data: T) -> ArenaIndex {
            let index = ArenaIndex(self.arena.len());
            let parent_idx = Some(parent_idx);
            self.arena.push((data, ArenaData { index, parent_idx }));

            index
        }
    }

    #[derive(Clone)]
    struct TestNode;

    impl Node for TestNode {
        type ChildrenIter = Vec<Self>;
        type Borrowable = Self;
        type Data = Option<()>;

        fn data(&self) -> &Self::Data {
            &None
        }
        fn parent(&self) -> Option<Self::Borrowable> {
            unimplemented!()
        }
        fn make_borrowable(&self) -> Self::Borrowable {
            unimplemented!()
        }
        fn children(&self) -> Self::ChildrenIter {
            Vec::new()
        }
        fn new_child(&self, state: Self::Data) -> Self {
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
