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
}

impl<T: GameState> MctsData<T> {
    pub fn increment_plays(&self) {
        self.plays.set(self.plays.get() + 1);
    }

    pub fn increment_wins(&self) {
        self.wins.set(self.wins.get() + 1);
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
        }
    }
}
