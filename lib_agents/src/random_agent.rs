use crate::util::random_choice;
use lib_boardgame::{GameAgent, GameState};

pub struct RandomAgent;

impl<TState: GameState> GameAgent<TState> for RandomAgent {
    fn pick_move(&self, _state: &TState, legal_moves: &[TState::Action]) -> TState::Action {
        random_choice(&legal_moves, &mut crate::util::get_rng())
    }
}
