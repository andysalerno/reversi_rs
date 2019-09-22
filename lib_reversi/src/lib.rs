pub mod reversi;
pub mod reversi_gamestate;
pub mod nboard_engine;
mod util;

use lib_boardgame::{GameMove, PlayerColor};
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
pub enum ReversiPlayerAction {
    PassTurn,
    Move { position: BoardPosition },
}

impl GameMove for ReversiPlayerAction {}
impl fmt::Debug for ReversiPlayerAction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let msg = match self {
            ReversiPlayerAction::PassTurn => "(player passes turn)".to_owned(),
            ReversiPlayerAction::Move { position } => {
                format!("({}, {})", position.col, position.row)
            }
        };

        write!(f, "{}", msg)
    }
}
impl fmt::Display for ReversiPlayerAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
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

impl std::str::FromStr for ReversiPlayerAction {
    type Err = usize;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.to_lowercase() == "pass" {
            return Ok(ReversiPlayerAction::PassTurn);
        }

        let nums: Vec<_> = s.split(',').map(|x| x.trim()).collect();

        if nums.len() != 2 {
            println!("Invalid input: {} -- expected format: col,row", s);
            return Err(9);
        }

        let col = nums[0].parse::<usize>();
        let row = nums[1].parse::<usize>();

        if let (Ok(col), Ok(row)) = (col, row) {
            let position = BoardPosition::new(col, row);
            if col > crate::reversi_gamestate::ReversiState::BOARD_SIZE
                || row >= crate::reversi_gamestate::ReversiState::BOARD_SIZE
            {
                println!(
                    "Position out of bounds of board. Input: {:?}, actual board size: {}",
                    position,
                    crate::reversi_gamestate::ReversiState::BOARD_SIZE
                );

                Err(9)
            } else {
                let action = ReversiPlayerAction::Move { position };
                Ok(action)
            }
        } else {
            println!("Didn't recognize input as a board position: {}", s);
            Err(9)
        }
    }
}
