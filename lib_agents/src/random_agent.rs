use lib_boardgame::{GameAgent, GameState};
use crate::util::random_choice;

pub struct RandomAgent;

impl RandomAgent {
    fn random_choice<T>(&self, choices: &[T]) -> T
    where
        T: Copy,
    {
        random_choice(choices)
    }
}

impl<TState: GameState> GameAgent<TState> for RandomAgent {
    fn pick_move(&self, _state: &TState, legal_moves: &[TState::Move]) -> TState::Move {
        self.random_choice(&legal_moves)
    }
}
