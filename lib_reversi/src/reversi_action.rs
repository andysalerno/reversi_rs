use crate::BoardPosition;
use lib_boardgame::GameMove;
use lib_printer::{out, out_impl};
use std::fmt;

#[derive(Copy, Clone, PartialEq)]
pub enum ReversiPlayerAction {
    PassTurn,
    Move { position: BoardPosition },
}

impl GameMove for ReversiPlayerAction {
    fn is_forced_pass(self) -> bool {
        match self {
            ReversiPlayerAction::PassTurn => true,
            _ => false,
        }
    }
}

impl fmt::Debug for ReversiPlayerAction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let msg = match self {
            ReversiPlayerAction::PassTurn => "(player passes turn)".to_owned(),
            ReversiPlayerAction::Move { position } => {
                format!("({}, {})", position.col(), position.row())
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

impl std::str::FromStr for ReversiPlayerAction {
    type Err = usize;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.to_lowercase() == "pass" {
            return Ok(ReversiPlayerAction::PassTurn);
        }

        let nums: Vec<_> = s.split(',').map(|x| x.trim()).collect();

        if nums.len() != 2 {
            out!("Invalid input: {} -- expected format: col,row", s);
            return Err(9);
        }

        let col = nums[0].parse::<usize>();
        let row = nums[1].parse::<usize>();

        if let (Ok(col), Ok(row)) = (col, row) {
            let position = BoardPosition::new(col, row);
            if col > crate::reversi_gamestate::ReversiState::BOARD_SIZE
                || row >= crate::reversi_gamestate::ReversiState::BOARD_SIZE
            {
                out!(
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
            out!("Didn't recognize input as a board position: {}", s);
            Err(9)
        }
    }
}
