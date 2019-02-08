/// This is a simple, generic reference-counted implementation of the Node trait.
use crate::tree::Node;
use std::cell::RefCell;
use std::rc::{Rc, Weak};

pub struct NodeContent<T> {
    data: T,
    parent: Weak<Self>,
    children: RefCell<Vec<RcNode<T>>>,
}

/// Wraps a NodeContent with a reference-counted owner.
type RcNode<T> = Rc<NodeContent<T>>;

impl<T: Clone> Node for RcNode<T>
where
    Self: Sized,
{
    type ChildrenIter = Vec<Self>;
    type ParentBorrow = Self;
    type Data = T;

    fn data(&self) -> &T {
        &self.data
    }

    fn parent(&self) -> Option<Self::ParentBorrow> {
        self.parent.upgrade().clone() // todo: clone() maybe not necessary?
    }

    fn children(&self) -> Self::ChildrenIter {
        let c: Vec<Self> = self.children.borrow().iter().map(|n| n.clone()).collect();

        c
    }

    fn add_child(&mut self, child: Self) {
        self.children.borrow_mut().push(child.clone());
    }

    fn new_child(&self, data: &T) -> RcNode<T> {
        let child = Rc::new(NodeContent {
            parent: Rc::downgrade(self),
            children: RefCell::default(),
            data: data.clone(),
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
mod test {
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
        let child = root.new_child(&TestData(2));

        let root_children = root.children();

        assert_eq!(root_children[0].data(), &TestData(2));
        assert_eq!(child.children().len(), 0);
    }

    #[test]
    fn child_has_parent() {
        let root = RcNode::new_root(TestData(1));
        let child = root.new_child(&TestData(2));

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
        let child_1 = root.new_child(&TestData(2));
        let child_2 = root.new_child(&TestData(3));
        let child_3 = root.new_child(&TestData(4));

        let child_4 = child_1.new_child(&TestData(5));
        let child_5 = child_2.new_child(&TestData(5));
        let child_6 = child_5.new_child(&TestData(5));

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

// #[cfg(test)]
// mod test {
//     use super::*;
//     use lib_boardgame::game_primitives::{GameMove, PlayerColor};

//     #[derive(Clone)]
//     struct TestGameState;

//     #[derive(Copy, Clone, Debug)]
//     struct TestGameAction;
//     impl GameMove for TestGameAction {}

//     impl GameState for TestGameState {
//         type Move = TestGameAction;

//         /// Returns a human-friendly string for representing the state.
//         fn human_friendly(&self) -> String {
//             unimplemented!()
//         }

//         /// Gives the implementation a chance to initialize the starting state of a game
//         /// before gameplay begins.
//         fn initialize_board(&mut self) {
//             unimplemented!()
//         }

//         /// Returns the possible moves the given player can make for the current state.
//         fn legal_moves(&self, _player: PlayerColor) -> Vec<Self::Move> {
//             unimplemented!()
//         }

//         /// Apply the given move (or 'action') to this state, mutating this state
//         /// and advancing it to the resulting state.
//         fn apply_move(&mut self, _action: Self::Move) {
//             unimplemented!()
//         }

//         /// Returns the current player whose turn it currently is.
//         fn current_player_turn(&self) -> PlayerColor {
//             unimplemented!()
//         }
//     }

//     /// Test that update_visit will update the wins
//     /// and plays count of the same node it is called on.
//     #[test]
//     fn test_update_visit() {
//         let state = TestGameState;
//         let root_node = RcNode::new_root(&state);

//         assert_eq!(0, root_node.data.wins());
//         assert_eq!(0, root_node.data.plays());

//         root_node.update_visit(1);

//         assert_eq!(1, root_node.data.wins());
//         assert_eq!(1, root_node.data.plays());

//         root_node.update_visit(0);

//         assert_eq!(1, root_node.data.wins());
//         assert_eq!(2, root_node.data.plays());
//     }

//     #[test]
//     fn back_prop_works() {
//         let state = TestGameState;
//         let tree = RcTree::new(state);

//         let state_p = TestGameState;
//         let action = TestGameAction;

//         // add some descendants to the parent root
//         let mut child_1 = RcNode::<TestGameState>::new_child(&mut tree.root(), action, &state_p);
//         let mut child_2 = RcNode::<TestGameState>::new_child(&mut child_1, action, &state_p);
//         let mut child_3 = RcNode::<TestGameState>::new_child(&mut child_2, action, &state_p);
//         let mut child_4 = RcNode::<TestGameState>::new_child(&mut child_3, action, &state_p);

//         // add two children directly to the bottom-most child
//         let left_5 = RcNode::<TestGameState>::new_child(&mut child_4, action, &state_p);
//         let right_5 = RcNode::<TestGameState>::new_child(&mut child_4, action, &state_p);

//         assert_eq!(0, child_1.wins());
//         assert_eq!(0, child_2.wins());
//         assert_eq!(0, child_3.wins());
//         assert_eq!(0, child_4.wins());
//         assert_eq!(0, left_5.wins());
//         assert_eq!(0, right_5.wins());

//         right_5.backprop(1);

//         assert_eq!(1, right_5.wins());
//         assert_eq!(1, right_5.plays());

//         assert_eq!(0, left_5.wins());
//         assert_eq!(0, left_5.plays());

//         assert_eq!(1, child_4.wins());
//         assert_eq!(1, child_4.plays());
//         assert_eq!(1, child_3.wins());
//         assert_eq!(1, child_3.plays());
//         assert_eq!(1, child_2.wins());
//         assert_eq!(1, child_2.plays());
//         assert_eq!(1, child_1.wins());
//         assert_eq!(1, child_1.plays());
//         assert_eq!(1, tree.root().wins());
//         assert_eq!(1, tree.root().plays());
//     }

//     #[test]
//     fn set_root_frees_unused_nodes() {
//         let state = TestGameState;
//         let mut tree = RcTree::new(state);

//         let state_p = TestGameState;
//         let action = TestGameAction;

//         // We create three levels of children, and capture the node
//         // at the third level, but the two higher-level nodes go out of scope...
//         let distant_child = {
//             let mut child_1 =
//                 RcNode::<TestGameState>::new_child(&mut tree.root(), action, &state_p);
//             let mut child_2 = RcNode::<TestGameState>::new_child(&mut child_1, action, &state_p);
//             let mut child_3 = RcNode::<TestGameState>::new_child(&mut child_2, action, &state_p);

//             // In this scope, upgrading to get the parent of child_3 should work.
//             assert!(child_3.parent().upgrade().is_some());

//             child_3
//         };

//         // In this scope, upgrading to get the parent of child_3 should STILL work,
//         // thanks to the magic of reference-counted storage.
//         assert!(distant_child.parent().upgrade().is_some());

//         // However, if we set distant_child as the new root of the tree,
//         // we should free up the nodes that are above distant_child (since it is the new root),
//         // and therefore we should not be able to upgrade to its parent anymore (since it doesn't exist).
//         tree.set_root(&distant_child);

//         // In this scope, upgrading to get the parent of child_3 should STILL work,
//         // thanks to the magic of reference-counted storage.
//         assert!(distant_child.parent().upgrade().is_none());
//     }
// }
