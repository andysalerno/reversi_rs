/// This describes the general Node trait that can be used for making trees (specifically, monte-carlo trees)
pub mod rc_tree;

use std::borrow::Borrow;

pub trait Node: Sized + Clone {
    type ChildrenIter: IntoIterator<Item = Self::Borrowable>;
    type Borrowable: Borrow<Self>;
    type Data;

    fn data(&self) -> &Self::Data;
    fn parent(&self) -> Option<Self::Borrowable>;
    fn make_borrowable(&self) -> Self::Borrowable;
    fn children(&self) -> Self::ChildrenIter;

    fn new_child(&self, state: Self::Data) -> Self::Borrowable;
    fn new_root(state: Self::Data) -> Self::Borrowable;
}

#[cfg(test)]
mod tests {
    use super::*;

    /* unfortunately, I discovered my arena impl is terrible at the last second, so I can't use these tests for now... */
    // mod arena_tests {
    //     use super::*;
    //     use std::cell::RefCell;
    //     use std::rc::Rc;

    //     thread_local! {
    //         static ARENA: Rc<RefCell<Arena<EmptyNodeData>>> = Default::default();
    //     }

    //     #[derive(Copy, Clone, Default)]
    //     struct ArenaIndex(usize);

    //     impl Borrow<ArenaNode<EmptyNodeData>> for ArenaIndex {
    //         fn borrow(&self) -> &ArenaNode<EmptyNodeData> {
    //             ARENA.with(|a| {
    //                 let rc_ptr: Rc<_> = a.clone();
    //                 let refcell: &RefCell<_> = rc_ptr.borrow();
    //                 let arena = refcell.borrow();

    //                 unsafe { arena.value.get_node(*self) }
    //             })
    //         }
    //     }

    //     #[derive(Default)]
    //     struct ArenaData {
    //         index: ArenaIndex,
    //         parent_idx: Option<ArenaIndex>,
    //     }

    //     #[derive(Default)]
    //     struct Arena<T> {
    //         arena: Vec<ArenaNode<T>>,
    //     }

    //     impl<T: Default> Arena<T> {
    //         fn new() -> Self {
    //             Self { arena: Vec::new() }
    //         }
    //     }

    //     impl<T> Arena<T> {
    //         fn get_node(&self, index: ArenaIndex) -> &ArenaNode<T> {
    //             &self.arena[index.0]
    //         }

    //         fn get_node_mut(&mut self, index: ArenaIndex) -> &mut ArenaNode<T> {
    //             &mut self.arena[index.0]
    //         }

    //         fn insert_root(&mut self, data: T) -> ArenaIndex {
    //             let index = ArenaIndex(self.arena.len());
    //             let parent_index = None;

    //             self.arena.push(ArenaNode {
    //                 index,
    //                 data,
    //                 parent_index,
    //                 children: Default::default(),
    //             });

    //             index
    //         }

    //         fn insert_node(&mut self, parent_idx: ArenaIndex, data: T) -> ArenaIndex {
    //             let index = ArenaIndex(self.arena.len());
    //             let parent_index = Some(parent_idx);
    //             self.arena.push(ArenaNode {
    //                 index,
    //                 data,
    //                 parent_index,
    //                 children: Default::default(),
    //             });

    //             index
    //         }
    //     }

    //     #[derive(Clone)]
    //     struct ArenaNode<T> {
    //         index: ArenaIndex,
    //         parent_index: Option<ArenaIndex>,
    //         children: Vec<ArenaIndex>,
    //         data: T,
    //     }

    //     #[derive(Default, Clone)]
    //     struct EmptyNodeData;

    //     impl Node for ArenaNode<EmptyNodeData> {
    //         type ChildrenIter = Vec<ArenaIndex>;
    //         type Borrowable = ArenaIndex;
    //         type Data = EmptyNodeData;

    //         fn data(&self) -> &Self::Data {
    //             &EmptyNodeData
    //         }

    //         fn parent(&self) -> Option<Self::Borrowable> {
    //             self.parent_index
    //         }

    //         fn make_borrowable(&self) -> Self::Borrowable {
    //             self.index
    //         }

    //         fn children(&self) -> Self::ChildrenIter {
    //             // todo: how do we avoid this clone?
    //             self.children.clone()
    //         }

    //         fn new_child(&self, state: Self::Data) -> ArenaIndex {
    //             ARENA.with(|a| {
    //                 let refcell: &RefCell<_> = a.borrow();
    //                 let mut arena = refcell.borrow_mut();

    //                 arena.insert_node(
    //                     self.parent_index
    //                         .expect("must have parent to insert a child."),
    //                     state,
    //                 )
    //             })
    //         }

    //         fn new_root(state: Self::Data) -> ArenaIndex {
    //             ARENA.with(|a| {
    //                 let refcell: &RefCell<_> = a.borrow();
    //                 let mut arena = refcell.borrow_mut();
    //                 arena.insert_root(state)
    //             })
    //         }
    //     }
    // }

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
