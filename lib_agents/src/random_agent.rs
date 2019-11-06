use crate::util::random_choice;
use lib_boardgame::{GameAgent, GameState, PlayerColor};

pub struct RandomAgent {
    player_color: PlayerColor,
}

impl<TState: GameState> GameAgent<TState> for RandomAgent {
    fn pick_move(&self, _state: &TState, legal_moves: &[TState::Move]) -> TState::Move {
        random_choice(&legal_moves, &mut crate::util::get_rng())
    }

    fn player_color(&self) -> PlayerColor {
        self.player_color
    }
}
