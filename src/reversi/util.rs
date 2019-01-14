use super::reversi_gamestate::BoardPosition;
use super::reversi_gamestate::Directions;
use super::reversi_gamestate::ReversiPiece;
use super::reversi_gamestate::ReversiState;

pub(super) fn opponent(piece: ReversiPiece) -> ReversiPiece {
    match piece {
        ReversiPiece::Black => ReversiPiece::White,
        ReversiPiece::White => ReversiPiece::Black,
    }
}

pub(super) struct BoardDirectionIter {
    origin: BoardPosition,
    direction: Directions,
    board_size: usize,

    /// for iteration -- what position are we currently at?
    cursor: BoardPosition,
}

impl BoardDirectionIter {
    pub fn new(origin: BoardPosition, direction: Directions) -> Self {
        if direction.col_dir == 0 && direction.row_dir == 0 {
            panic!("Can't create an iterator with both column and row direction as 0 (this would result in an iterator that never moves)");
        }

        BoardDirectionIter {
            origin,
            direction,
            board_size: ReversiState::BoardSize,

            cursor: origin,
        }
    }
}

impl Iterator for BoardDirectionIter {
    type Item = BoardPosition;

    fn next(&mut self) -> Option<Self::Item> {
        let next_col = self.cursor.col as i32 + self.direction.col_dir;
        let next_row = self.cursor.row as i32 + self.direction.row_dir;

        if next_col < 0 || next_row < 0 {
            return None;
        }

        if next_col >= self.board_size as i32 || next_row >= self.board_size as i32 {
            return None;
        }

        self.cursor.col = next_col as usize;
        self.cursor.row = next_row as usize;

        let next_pos = BoardPosition::new(next_col as usize, next_row as usize);

        Some(next_pos)
    }
}