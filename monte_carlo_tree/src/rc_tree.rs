use super::Node;
use lib_boardgame::game_primitives::GameState;
use std::rc::{Rc, Weak};

struct BoxNode<T: GameState> {
    state: T,
    parent: Weak<BoxNode<T>>,
    children: Vec<Rc<BoxNode<T>>>,
}

impl<T: GameState> BoxNode<T> {
    fn new_child(parent: &Rc<Self>, state: &T) -> Self {
        BoxNode {
            parent: Rc::downgrade(parent),
            children: Vec::new(),
            state: state.clone(),
        }
    }

    fn new_root(state: &T) -> Self {
        BoxNode {
            parent: Weak::new(),
            children: Vec::new(),
            state: state.clone(),
        }
    }

    fn plays(&self) -> usize {
        unimplemented!()
    }
    fn wins(&self) -> usize {
        unimplemented!()
    }
    fn losses(&self) -> usize {
        unimplemented!()
    }
    fn parent(&self) -> Weak<Self> {
        self.parent.clone()
    }
    fn children(&self) -> &[Rc<Self>] {
        &self.children
    }
}

struct BoxTree<T: GameState> {
    root: BoxNode<T>,
}

impl<T: GameState> BoxTree<T> {
    fn new(game_state: T) -> Self {
        let root = BoxNode::new_root(&game_state);

        BoxTree { root }
    }
}
