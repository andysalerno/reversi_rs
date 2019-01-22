use lib_boardgame::game_primitives::GameState;
use std::rc::{Rc, Weak};

pub struct RcNode<T: GameState> {
    state: T,
    action: Option<T::Move>,
    parent: Weak<RcNode<T>>,
    children: Vec<Rc<Self>>,

    plays: usize,
    wins: usize,
    losses: usize,
}

impl<T: GameState> RcNode<T> {
    fn new_child(parent: &mut Rc<Self>, action: T::Move, state: &T) -> Rc<Self> {
        let child = Rc::new(Self {
            parent: Rc::downgrade(parent),
            children: Vec::new(),
            state: state.clone(),
            action: Some(action),
            plays: 0,
            wins: 0,
            losses: 0,
        });

        dbg!(Rc::get_mut(parent)
            .expect("Couldn't access parent of new child as mutable.")
            .add_child(&child));

        child
    }

    fn new_root(state: &T) -> Self {
        Self {
            parent: Weak::new(),
            children: Vec::new(),
            state: state.clone(),
            action: None,
            plays: 0,
            wins: 0,
            losses: 0,
        }
    }

    fn plays(&self) -> usize {
        self.plays
    }
    fn wins(&self) -> usize {
        self.wins
    }
    fn losses(&self) -> usize {
        self.losses
    }
    fn parent(&self) -> Weak<Self> {
        self.parent.clone()
    }
    fn children(&self) -> &[Rc<Self>] {
        &self.children
    }
    fn add_child(&mut self, child: &Rc<Self>) {
        self.children.push(child.clone());
    }
    fn action(&self) -> T::Move {
        self.action.unwrap()
    }

    fn state(&self) -> &T {
        &self.state
    }

    fn update_visit(&mut self, delta: usize) {
        self.plays += 1;
        self.wins += delta;
    }
}

pub struct BoxTree<T: GameState> {
    root: Rc<RcNode<T>>,
}

impl<T: GameState> BoxTree<T> {
    pub fn new(game_state: T) -> Self {
        let root = RcNode::new_root(&game_state);

        BoxTree {
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
            .iter()
            .max_by_key(|c| c.wins())
            .unwrap()
            .action()
    }

    /// From the set of child nodes of the current node,
    /// select the one whose subtree we will explore.
    fn select(&self) -> Rc<RcNode<T>> {
        let selected = &self.root.children[0];
        selected.clone()
    }

    fn backprop(node: &mut RcNode<T>, delta: usize) {
        node.update_visit(delta);

        loop {
            if let Some(mut n) = node.parent().upgrade() {
                Rc::get_mut(&mut n)
                    .expect("couldn't get mutable reference to parent node")
                    .update_visit(delta);
            } else {
                // If we can't get the parent, we must be at the root.
                break;
            }
        }
    }

    fn set_root(&mut self, new_root: Rc<RcNode<T>>) {
        self.root = new_root.clone();
    }

    fn root(&self) -> Rc<RcNode<T>> {
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

    #[test]
    fn back_prop_works() {
        let state = TestGameState;
        let tree = BoxTree::new(state);

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
    }
}
