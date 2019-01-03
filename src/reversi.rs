use crate::game_primitives::{Game, GameState, GameMove, PlayerColor};

#[derive(Copy, Clone)]
struct ReversiMove;
impl GameMove for ReversiMove {}

#[derive(Clone)]
struct ReversiState;

impl GameState for ReversiState {
    type Move = ReversiMove;

    /// Returns a human-friendly string for representing the state.
    fn human_friendly(&self) -> String {
        "hello".to_owned()
    }

    /// Returns the possible moves the given player can make for the current state.
    fn legal_moves(&self, player: PlayerColor) -> Vec<Self::Move> {
        Vec::new()
    }

    /// Apply the given move (or 'action') to this state, mutating this state
    /// and advancing it to the resulting state.
    fn apply_move(&mut self, action: Self::Move) {}
}

struct Reversi;

impl Game for Reversi {}
