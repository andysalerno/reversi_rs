use lib_boardgame::game_primitives::GameState;
use std::cell::Cell;

pub trait Data<T: GameState> {
    fn state(&self) -> &T;
    fn plays(&self) -> usize;
    fn wins(&self) -> usize;
    fn action(&self) -> Option<T::Move>;
    fn new(state: &T, plays: usize, wins: usize, action: Option<T::Move>) -> Self;
}

/// MCTS-related data that every Node will have.
#[derive(Default, Clone)]
pub struct MctsData<T: GameState> {
    state: T,
    plays: Cell<usize>,
    wins: Cell<usize>,
    action: Option<T::Move>,

    children_count: Cell<usize>,
    children_saturated_count: Cell<usize>,
}

impl<T: GameState> MctsData<T> {
    pub fn increment_plays(&self) {
        self.plays.set(self.plays.get() + 1);
    }

    pub fn increment_wins(&self) {
        self.wins.set(self.wins.get() + 1);
    }

    /// A node is considered saturated if:
    ///     * it is a leaf node (has no children), OR
    ///     * every one of its children is saturated
    /// During MCTS, we should not traverse down saturated nodes,
    /// since we have already seen every outcome.
    /// Nodes should not be marked saturated until AFTER their result
    /// has been backpropagated.
    pub fn is_saturated(&self) -> bool {
        self.children_count == self.children_saturated_count
    }

    pub fn set_children_count(&self, count: usize) {
        self.children_count.set(count);
    }

    pub fn increment_saturated_children_count(&self) {
        self.children_saturated_count
            .set(self.children_saturated_count.get() + 1);

        assert!(self.children_saturated_count.get() <= self.children_count.get());
    }
}

impl<T: GameState> Data<T> for MctsData<T> {
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
            action,
            children_count: Default::default(),
            children_saturated_count: Default::default(),
        }
    }
}
