use crate::game_primitives::{GameAgent, GameState, PlayerColor};

pub struct RandomAgent {
    color: PlayerColor,
}

impl RandomAgent {
    /// Creates a new RandomAgent, playing for the given color.
    pub fn new(color: PlayerColor) -> Self {
        RandomAgent { color }
    }
}

impl<TState: GameState> GameAgent<TState> for RandomAgent {
    fn pick_move(&self, state: &TState) -> TState::Move {
        // not very random yet :)
        state.legal_moves(self.color)[0]
    }
}
