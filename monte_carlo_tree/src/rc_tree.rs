use lib_boardgame::game_primitives::GameState;
use std::rc::{Rc, Weak};

pub struct BoxNode<T: GameState> {
    state: T,
    action: Option<T::Move>,
    parent: Weak<BoxNode<T>>,
    children: Vec<Rc<Self>>,

    plays: usize,
    wins: usize,
    losses: usize,
}

impl<T: GameState> BoxNode<T> {
    fn new_child(parent: &Rc<Self>, action: T::Move, state: &T) -> Self {
        BoxNode {
            parent: Rc::downgrade(parent),
            children: Vec::new(),
            state: state.clone(),
            action: Some(action),
            plays: 0,
            wins: 0,
            losses: 0,
        }
    }

    fn new_root(state: &T) -> Self {
        BoxNode {
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
    fn action(&self) -> T::Move {
        self.action.unwrap()
    }

    fn state(&self) -> &T {
        &self.state
    }
}

pub struct BoxTree<T: GameState> {
    root: Rc<BoxNode<T>>,
}

impl<T: GameState> BoxTree<T> {
    pub fn new(game_state: T) -> Self {
        let root = BoxNode::new_root(&game_state);

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
    fn select(&self) -> Rc<BoxNode<T>> {
        let selected = &self.root.children[0];
        selected.clone()
    }

    fn set_root(&mut self, new_root: Rc<BoxNode<T>>) {
        self.root = new_root.clone();
    }
}
