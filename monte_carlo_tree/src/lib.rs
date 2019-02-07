use lib_boardgame::game_primitives::GameState;
use std::borrow::Borrow;
use std::cell::Cell;

pub mod rc_tree;

pub trait Data<T: GameState> {
    fn state(&self) -> &T;
    fn plays(&self) -> usize;
    fn wins(&self) -> usize;
    fn action(&self) -> Option<T::Move>;
    fn new(state: &T, plays: usize, wins: usize, action: Option<T::Move>) -> Self;
}

/// MCTS-related data that every Node will have.
#[derive(Default)]
pub struct NodeData<T: GameState> {
    state: T,
    plays: Cell<usize>,
    wins: Cell<usize>,
    action: Option<T::Move>,
}

impl<T: GameState> Data<T> for NodeData<T> {
    fn state(&self) -> &T {
        &self.state
    }

    fn plays(&self) -> usize {
        self.plays.get()
    }

    fn wins(&self) -> usize {
        self.wins.get()
    }

    fn action(&self) -> Option<T::Move> {
        self.action
    }

    fn new(state: &T, plays: usize, wins: usize, action: Option<T::Move>) -> Self {
        Self {
            state: state.clone(),
            plays: Cell::new(plays),
            wins: Cell::new(wins),
            action: action.clone(),
        }
    }
}

pub trait Node
where
    Self: Sized,
{
    type ChildrenIter: IntoIterator<Item = Self>;
    type ParentBorrow: Borrow<Self>;
    type TState: GameState;

    fn data(&self) -> &NodeData<Self::TState>;
    fn parent(&self) -> Option<Self::ParentBorrow>;
    fn children(&self) -> Self::ChildrenIter;
    fn add_child(&mut self, child: Self);

    fn new_child(
        &self,
        action: <<Self as Node>::TState as GameState>::Move,
        state: &Self::TState,
    ) -> Self;
    fn new_root(state: &Self::TState) -> Self;
}

pub struct MonteCarloTree<N: Node> {
    root: N,
}

impl<N: Node> MonteCarloTree<N> {
    pub fn new(game_state: &N::TState) -> Self {
        Self {
            root: N::new_root(game_state),
        }
    }

    pub fn select_child() {
        unimplemented!()
    }

    pub fn expand() {
        unimplemented!()
    }

    pub fn simulate() {
        unimplemented!()
    }

    pub fn backprop() {
        unimplemented!()
    }

    fn choose_best_action() {
        // self.root
        //     .children()
        //     .iter()
        //     .max_by_key(|c| c.data.wins())
        //     .unwrap()
        //     .data
        //     .action()
        //     .unwrap()
    }
}
