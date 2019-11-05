use lib_boardgame::PlayerColor;

/// The size of the board.
/// E.x., if this is 8, the Reversi board is 8x8 spaces large.
pub(crate) const BOARD_SIZE: usize = 8;

pub(crate) type Board = [[Option<ReversiPiece>; BOARD_SIZE]; BOARD_SIZE];

/// When traversing pieces on the board,
/// a positive direction indicates increasing values for col or row,
/// a negative direction indicates decreasing values for col or row,
/// and a 'same' direction indicates no movement for col or row.
/// Example: if we ask to traverse as 'col: positive, row: negative',
/// our traversal will increment with increasing col values, whereas row will be decremented.
/// (I.e., down and to the right.)
pub(crate) mod board_directions {
    pub type Direction = i32;
    pub const POSITIVE: Direction = 1;
    pub const NEGATIVE: Direction = -1;
    pub const SAME: Direction = 0;
}

#[derive(Copy, Clone)]
pub(crate) struct Directions {
    pub col_dir: board_directions::Direction,
    pub row_dir: board_directions::Direction,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ReversiPiece {
    Black,
    White,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct BoardPosition {
    pub(crate) col: usize,
    pub(crate) row: usize,
}

impl BoardPosition {
    pub fn new(col: usize, row: usize) -> Self {
        Self { col, row }
    }

    pub fn col(&self) -> usize {
        self.col
    }

    pub fn row(&self) -> usize {
        self.row
    }
}

impl From<PlayerColor> for ReversiPiece {
    fn from(color: PlayerColor) -> ReversiPiece {
        match color {
            PlayerColor::Black => ReversiPiece::Black,
            PlayerColor::White => ReversiPiece::White,
        }
    }
}
