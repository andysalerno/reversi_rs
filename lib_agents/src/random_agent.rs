use lib_boardgame::{GameAgent, GameState};
use crate::util::random_choice;

pub struct RandomAgent;

impl<TState: GameState> GameAgent<TState> for RandomAgent {
    fn pick_move(&self, _state: &TState, legal_moves: &[TState::Move]) -> TState::Move {
        random_choice(&legal_moves, &mut crate::util::get_rng())
    }
}
