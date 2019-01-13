use super::reversi_gamestate::BoardPosition;
use super::reversi_gamestate::Directions;
use super::reversi_gamestate::PiecePos;
use super::reversi_gamestate::ReversiPiece;
use super::reversi_gamestate::ReversiState;

pub(super) fn opponent(piece: ReversiPiece) -> ReversiPiece {
    match piece {
        ReversiPiece::Black => ReversiPiece::White,
        ReversiPiece::White => ReversiPiece::Black,
    }
}

pub(super) struct BoardDirectionIter<'a> {
    game_state: &'a ReversiState,
    origin: BoardPosition,
    direction: Directions,
}

impl<'a> BoardDirectionIter<'a> {
    pub fn new(game_state: &'a ReversiState, origin: BoardPosition, direction: Directions) -> Self {
        BoardDirectionIter {
            game_state,
            origin,
            direction,
        }
    }
}

impl<'a> Iterator for BoardDirectionIter<'a> {
    type Item = PiecePos;

    fn next(&mut self) -> Option<Self::Item> {
        let piece_pos: Self::Item = (self.game_state.get_piece(self.origin).unwrap(), self.origin);
        Some(piece_pos)
    }
}
