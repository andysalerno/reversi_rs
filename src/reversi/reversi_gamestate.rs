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

impl ReversiState {
    /// Given an (x,y) coord within range of the board, return the ReversiPiece
    /// present on that spot, or None if the position is empty.
    /// Note: (0,0) is the bottom-left position.
    fn piece_at(&self, col: usize, row: usize) -> Option<ReversiPiece> {
        let row_prime = BOARD_SIZE - row - 1;

        self.board[row_prime][col]
    }
}

impl GameState for ReversiState {
    type Move = ReversiMove;

    /// Returns a human-friendly string for representing the state.
    fn human_friendly(&self) -> String {
        let mut result = String::new();

        const BLACK_PIECE: char = 'X';
        const WHITE_PIECE: char = 'O';
        const EMPTY_SPACE: char = '-';

        result.push('\n');

        for row in (0..BOARD_SIZE).rev() {
            result.push_str("| ");

            for col in (0..BOARD_SIZE) {
                let piece = self.piece_at(col, row);

                let piece_char = match piece {
                    Some(ReversiPiece::White) => WHITE_PIECE,
                    Some(ReversiPiece::Black) => BLACK_PIECE,
                    None => EMPTY_SPACE, 
                };

                result.push(piece_char);
                result.push(' ');
            }

            result.push('\n');
        }

        result.push(' ');
        for _ in (0..BOARD_SIZE) {
            result.push_str("--");
        }

        result
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

        println!("{}", stringified);

        assert_eq!(2 + 2, 4);
    }
}
