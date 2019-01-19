use lib_boardgame::game_primitives::{GameAgent, GameState};
use rand::seq::SliceRandom;

pub struct RandomAgent;

impl RandomAgent {
    fn random_choice<T>(&self, choices: &[T]) -> T
    where
        T: Copy,
    {
        *choices.choose(&mut rand::thread_rng()).unwrap()
    }
}

impl<TState: GameState> GameAgent<TState> for RandomAgent {
    fn pick_move(&self, _state: &TState, legal_moves: &[TState::Move]) -> TState::Move {
        self.random_choice(&legal_moves)
    }
}
