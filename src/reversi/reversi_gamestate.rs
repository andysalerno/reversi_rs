use crate::game_primitives::{GameMove, GameState, PlayerColor};

/// The size of the board.
/// E.x., if this is 8, the Reversi board is 8x8 spaces large.
const BOARD_SIZE: usize = 8;

#[derive(Copy, Clone)]
pub struct ReversiMove;
impl GameMove for ReversiMove {}

#[derive(Copy, Clone)]
enum ReversiPiece {
    Black,
    White,
}

type Board = [[Option<ReversiPiece>; BOARD_SIZE]; BOARD_SIZE];

#[derive(Clone)]
pub struct ReversiState {
    /// The underlying 2d array of board pieces.
    board: Board,

    /// The player whose turn it currently is.
    current_player_turn: PlayerColor,
}

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

    /// Returns the current player whose turn it currently is.
    fn current_player_turn(&self) -> PlayerColor {
        self.current_player_turn
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        use super::{Board, GameState, PlayerColor, ReversiState, BOARD_SIZE};

        let board: Board = [[None; BOARD_SIZE]; BOARD_SIZE];
        let state = ReversiState {
            board,
            current_player_turn: PlayerColor::Black,
        };

        let stringified = state.human_friendly();

        assert_eq!(2 + 2, 4);
    }
}
