use lib_boardgame::game_primitives::GameState;
use std::cell::Cell;
use std::cell::RefCell;
use std::rc::{Rc, Weak};

pub struct RcNode<T: GameState> {
    state: T,
    action: Option<T::Move>,
    parent: Weak<RcNode<T>>,
    children: RefCell<Vec<Rc<Self>>>,

    plays: Cell<usize>,
    wins: Cell<usize>,
}

impl<T: GameState> RcNode<T> {
    fn new_child(parent: &mut Rc<Self>, action: T::Move, state: &T) -> Rc<Self> {
        let child = Rc::new(Self {
            parent: Rc::downgrade(parent),
            children: RefCell::new(Vec::new()),
            state: state.clone(),
            action: Some(action),
            plays: Cell::new(0),
            wins: Cell::new(0),
        });

        dbg!(parent.add_child(&child));

        child
    }

    fn new_root(state: &T) -> Self {
        Self {
            parent: Weak::new(),
            children: RefCell::new(Vec::new()),
            state: state.clone(),
            action: None,
            plays: Cell::new(0),
            wins: Cell::new(0),
        }
    }

    pub fn plays(&self) -> usize {
        self.plays.get()
    }

    pub fn wins(&self) -> usize {
        self.wins.get()
    }

    pub fn parent(&self) -> Weak<Self> {
        self.parent.clone()
    }

    pub fn children(&self) -> &RefCell<Vec<Rc<Self>>> {
        &self.children
    }

    fn add_child(&self, child: &Rc<Self>) {
        self.children.borrow_mut().push(child.clone());
    }

    fn action(&self) -> T::Move {
        self.action.unwrap()
    }

    pub fn state(&self) -> &T {
        &self.state
    }

    fn update_visit(&self, delta: usize) {
        self.plays.set(self.plays() + 1);
        self.wins.set(self.wins() + delta);
    }

    fn backprop(&self, delta: usize) {
        // update this node's values
        self.update_visit(delta);

        let mut node = if let Some(n) = self.parent().upgrade() {
            n
        } else {
            return;
        };

        loop {
            node.update_visit(delta);

            if let Some(n) = node.parent().upgrade() {
                node = n.clone();
            } else {
                // If we can't get the parent, we must be at the root.
                break;
            }
        }
    }
}

pub struct RcTree<T: GameState> {
    root: Rc<RcNode<T>>,
}

impl<T: GameState> RcTree<T> {
    pub fn new(game_state: T) -> Self {
        let root = RcNode::new_root(&game_state);

        Self {
            root: Rc::new(root),
        }
    }

    /// Returns the MCTS herustic's top choice for
    /// which action to take while in the current root node's
    /// state.  TODO: currently, this is chosen by most wins,
    /// which is not optimal MCTS heuristic.
    pub fn choose_best_action(&self) -> T::Move {
        self.root
            .children()
            .borrow()
            .iter()
            .max_by_key(|c| c.wins())
            .unwrap()
            .action()
    }

    /// From the set of child nodes of the current node,
    /// select the one whose subtree we will explore.
    fn select(&self) -> Rc<RcNode<T>> {
        let selected = &self.root.children.borrow()[0];
        selected.clone()
    }

    fn set_root(&mut self, new_root: &Rc<RcNode<T>>) {
        self.root = new_root.clone();
    }

    pub fn root(&self) -> Rc<RcNode<T>> {
        self.root.clone()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use lib_boardgame::game_primitives::{GameMove, PlayerColor};

    #[derive(Clone)]
    struct TestGameState;

    #[derive(Copy, Clone, Debug)]
    struct TestGameAction;
    impl GameMove for TestGameAction {}

    impl GameState for TestGameState {
        type Move = TestGameAction;

        /// Returns a human-friendly string for representing the state.
        fn human_friendly(&self) -> String {
            unimplemented!()
        }

        /// Gives the implementation a chance to initialize the starting state of a game
        /// before gameplay begins.
        fn initialize_board(&mut self) {
            unimplemented!()
        }

        /// Returns the possible moves the given player can make for the current state.
        fn legal_moves(&self, _player: PlayerColor) -> Vec<Self::Move> {
            unimplemented!()
        }

        /// Apply the given move (or 'action') to this state, mutating this state
        /// and advancing it to the resulting state.
        fn apply_move(&mut self, _action: Self::Move) {
            unimplemented!()
        }

        /// Returns the current player whose turn it currently is.
        fn current_player_turn(&self) -> PlayerColor {
            unimplemented!()
        }
    }

    /// Test that update_visit will update the wins
    /// and plays count of the same node it is called on.
    #[test]
    fn test_update_visit() {
        let state = TestGameState;
        let tree = RcTree::new(state);
        let root_node = tree.root();

        assert_eq!(0, root_node.wins());
        assert_eq!(0, root_node.plays());

        root_node.update_visit(1);

        assert_eq!(1, root_node.wins());
        assert_eq!(1, root_node.plays());

        root_node.update_visit(0);

        assert_eq!(1, root_node.wins());
        assert_eq!(2, root_node.plays());
    }

    #[test]
    fn back_prop_works() {
        let state = TestGameState;
        let tree = RcTree::new(state);

        let state_p = TestGameState;
        let action = TestGameAction;

        // add some descendants to the parent root
        let mut child_1 = RcNode::<TestGameState>::new_child(&mut tree.root(), action, &state_p);
        let mut child_2 = RcNode::<TestGameState>::new_child(&mut child_1, action, &state_p);
        let mut child_3 = RcNode::<TestGameState>::new_child(&mut child_2, action, &state_p);
        let mut child_4 = RcNode::<TestGameState>::new_child(&mut child_3, action, &state_p);

        // add two children directly to the bottom-most child
        let left_5 = RcNode::<TestGameState>::new_child(&mut child_4, action, &state_p);
        let right_5 = RcNode::<TestGameState>::new_child(&mut child_4, action, &state_p);

        assert_eq!(0, child_1.wins());
        assert_eq!(0, child_2.wins());
        assert_eq!(0, child_3.wins());
        assert_eq!(0, child_4.wins());
        assert_eq!(0, left_5.wins());
        assert_eq!(0, right_5.wins());

        right_5.backprop(1);

        assert_eq!(1, right_5.wins());
        assert_eq!(1, right_5.plays());

        assert_eq!(0, left_5.wins());
        assert_eq!(0, left_5.plays());

        assert_eq!(1, child_4.wins());
        assert_eq!(1, child_4.plays());
        assert_eq!(1, child_3.wins());
        assert_eq!(1, child_3.plays());
        assert_eq!(1, child_2.wins());
        assert_eq!(1, child_2.plays());
        assert_eq!(1, child_1.wins());
        assert_eq!(1, child_1.plays());
        assert_eq!(1, tree.root().wins());
        assert_eq!(1, tree.root().plays());
    }

    #[test]
    fn set_root_frees_unused_nodes() {
        let state = TestGameState;
        let mut tree = RcTree::new(state);

        let state_p = TestGameState;
        let action = TestGameAction;

        // We create three levels of children, and capture the node
        // at the third level, but the two higher-level nodes go out of scope...
        let distant_child = {
            let mut child_1 =
                RcNode::<TestGameState>::new_child(&mut tree.root(), action, &state_p);
            let mut child_2 = RcNode::<TestGameState>::new_child(&mut child_1, action, &state_p);
            let mut child_3 = RcNode::<TestGameState>::new_child(&mut child_2, action, &state_p);

            // In this scope, upgrading to get the parent of child_3 should work.
            assert!(child_3.parent().upgrade().is_some());

            child_3
        };

        // In this scope, upgrading to get the parent of child_3 should STILL work,
        // thanks to the magic of reference-counted storage.
        assert!(distant_child.parent().upgrade().is_some());

        // However, if we set distant_child as the new root of the tree,
        // we should free up the nodes that are above distant_child (since it is the new root),
        // and therefore we should not be able to upgrade to its parent anymore (since it doesn't exist).
        tree.set_root(&distant_child);

        // In this scope, upgrading to get the parent of child_3 should STILL work,
        // thanks to the magic of reference-counted storage.
        assert!(distant_child.parent().upgrade().is_none());
    }
}
