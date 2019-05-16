pub mod reversi;
pub mod reversi_gamestate;
mod util;

use lib_boardgame::game_primitives::GameMove;
use std::fmt;

/// The size of the board.
/// E.x., if this is 8, the Reversi board is 8x8 spaces large.
/// TODO: put this in lib.rs
const BOARD_SIZE: usize = 8;

type Board = [[Option<ReversiPiece>; BOARD_SIZE]; BOARD_SIZE];

/// When traversing pieces on the board,
/// a positive direction indicates increasing values for col or row,
/// a negative direction indicates decreasing values for col or row,
/// and a 'same' direction indicates no movement for col or row.
/// Example: if we ask to traverse as 'col: positive, row: negative',
/// our traversal will increment with increasing col values, whereas row will be decremented.
/// (I.e., down and to the right.)
mod board_directions {
    pub type Direction = i32;
    pub const POSITIVE: Direction = 1;
    pub const NEGATIVE: Direction = -1;
    pub const SAME: Direction = 0;
}

#[derive(Copy, Clone)]
struct Directions {
    col_dir: board_directions::Direction,
    row_dir: board_directions::Direction,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ReversiPiece {
    Black,
    White,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct BoardPosition {
    col: usize,
    row: usize,
}

impl BoardPosition {
    fn new(col: usize, row: usize) -> Self {
        Self { col, row }
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum ReversiAction {
    PassTurn,
    Move {
        piece: ReversiPiece,
        position: BoardPosition,
    },
}

impl GameMove for ReversiAction {}
impl fmt::Debug for ReversiAction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let msg = match self {
            ReversiAction::PassTurn => "(player passes turn)".to_owned(),
            ReversiAction::Move { piece, position } => {
                format!("({}, {}, {:?})", position.col, position.row, piece)
            }
        };

        write!(f, "{}", msg)
    }
}

#[cfg(test)]
mod tests {
    use lib_agents::random_agent::RandomAgent;
    use crate::reversi::Reversi;
    use lib_boardgame::game_primitives::{Game};

    #[test]
    fn create_game() {
        let white = RandomAgent;
        let black = RandomAgent;

        let mut game = Reversi::new(white, black);
        game.play_to_end();
    }
}
