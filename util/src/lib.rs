use lib_reversi::{reversi_gamestate::ReversiState, BoardPosition, Directions};
use lib_boardgame::game_primitives::PlayerColor;
use rand::seq::SliceRandom;

pub(crate) struct BoardDirectionIter {
    direction: Directions,
    board_size: usize,

    /// for iteration -- what position are we currently at?
    cursor: BoardPosition,
}

pub(crate) fn opponent(player: PlayerColor) -> PlayerColor {
    match player {
        PlayerColor::Black => PlayerColor::White,
        PlayerColor::White => PlayerColor::Black,
    }
}

pub(crate) fn random_choice<T>(choices: &[T]) -> T
where
    T: Copy,
{
    *random_pick(choices)
}

pub(crate) fn random_pick<'a, T>(choices: &'a[T]) -> &'a T {
    choices.choose(&mut rand::thread_rng()).unwrap()
}

impl BoardDirectionIter {
    pub fn new(origin: BoardPosition, direction: Directions) -> Self {
        if direction.col_dir == 0 && direction.row_dir == 0 {
            panic!("Can't create an iterator with both column and row direction as 0 (this would result in an iterator that never moves)");
        }

        BoardDirectionIter {
            direction,
            board_size: ReversiState::BOARD_SIZE,

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
