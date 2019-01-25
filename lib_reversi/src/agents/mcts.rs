use lib_boardgame::game_primitives::{GameAgent, GameState};

pub struct MCTSAgent;

impl<TState: GameState> GameAgent<TState> for MCTSAgent {
    fn pick_move(&self, _state: &TState, legal_moves: &[TState::Move]) -> TState::Move {
        unimplemented!();
    }
}
