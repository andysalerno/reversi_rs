pub mod tic_tac_toe;
pub mod tic_tac_toe_gamestate;

use lib_boardgame::PlayerColor;

/// The size of the board.  E.x. if 3, the board is a 3x3 grid.
pub const BOARD_SIZE: usize = 3;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TicTacToePiece {
    X,
    O,
} 

impl TicTacToePiece {
    pub fn player_color(self) -> PlayerColor {
        match self {
            TicTacToePiece::X => PlayerColor::Black,
            TicTacToePiece::O => PlayerColor::White,
        }
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
