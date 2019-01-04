use crate::game_primitives::{GameAgent, GameState, PlayerColor};

struct RandomAgent {
    color: PlayerColor,
}

impl<TState: GameState> GameAgent<TState> for RandomAgent {
    fn pick_move(&self, state: &TState) -> TState::Move {
        // not very random yet :)
        state.legal_moves(self.color)[0]
    }
}
