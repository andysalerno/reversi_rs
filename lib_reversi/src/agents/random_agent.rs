use lib_boardgame::game_primitives::{GameAgent, GameState, PlayerColor};
use rand::seq::SliceRandom;

pub struct RandomAgent {
    color: PlayerColor,
}

impl RandomAgent {
    /// Creates a new RandomAgent, playing for the given color.
    pub fn new(color: PlayerColor) -> Self {
        RandomAgent {
            color,
        }
    }

    fn random_choice<T>(&self, choices: &[T]) -> T
    where
        T: Copy,
    {
        *choices.choose(&mut rand::thread_rng()).unwrap()
    }
}

impl<TState: GameState> GameAgent<TState> for RandomAgent {
    fn pick_move(&self, state: &TState) -> TState::Move {
        let legal_moves = dbg!(state.legal_moves(self.color));

        self.random_choice(&legal_moves)
    }
}
