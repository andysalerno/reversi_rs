use super::Node;
use lib_boardgame::game_primitives::GameState;
use std::rc::{Rc, Weak};

struct BoxNode<T: GameState> {
    state: T,
    parent: Weak<BoxNode<T>>,
    children: Vec<Rc<BoxNode<T>>>,

    plays: usize,
    wins: usize,
    losses: usize,
}

impl<T: GameState> BoxNode<T> {
    fn new_child(parent: &Rc<Self>, state: &T) -> Self {
        BoxNode {
            parent: Rc::downgrade(parent),
            children: Vec::new(),
            state: state.clone(),
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
}

struct BoxTree<T: GameState> {
    root: Rc<BoxNode<T>>,
}

impl<T: GameState> BoxTree<T> {
    fn new(game_state: T) -> Self {
        let root = BoxNode::new_root(&game_state);

        BoxTree {
            root: Rc::new(root),
        }
    }

    /// From the set of child nodes of the current node,
    /// select the one whose subtree we will explore.
    fn select() {}

    fn set_root(&mut self, new_root: Rc<BoxNode<T>>) {
        self.root = new_root.clone();
    }
}
