use super::util;
use crate::game_primitives::{GameMove, GameState, PlayerColor};

/// The size of the board.
/// E.x., if this is 8, the Reversi board is 8x8 spaces large.
const BOARD_SIZE: usize = 8;

/// When traversing pieces on the board,
/// a positive direction indicates increasing values for col or row,
/// a negative direction indicates decreasing values for col or row,
/// and a 'same' direction indicates no movement for col or row.
/// Example: if we ask to traverse as 'col: positive, row: negative',
/// our traversal will increment with increasing col values, whereas row will be decremented.
/// (I.e., down and to the right.)
type Direction = i32;
const POSITIVE: Direction = 1;
const NEGATIVE: Direction = -1;
const SAME: Direction = 0;

#[derive(Copy, Clone)]
pub(super) struct Directions {
    col_dir: Direction,
    row_dir: Direction,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub(super) enum ReversiPiece {
    Black,
    White,
}

#[derive(Copy, Clone, Debug)]
pub(super) struct BoardPosition {
    col: usize,
    row: usize,
}

impl BoardPosition {
    fn new(col: usize, row: usize) -> Self {
        Self { col, row }
    }
}

#[derive(Copy, Clone)]
pub struct ReversiMove {
    /// The piece to be placed at the given location.
    piece: ReversiPiece,

    /// The position at which to place the piece.
    position: BoardPosition,
}
impl GameMove for ReversiMove {}

pub(super) type Board = [[Option<ReversiPiece>; BOARD_SIZE]; BOARD_SIZE];
pub(super) type PiecePos = (ReversiPiece, BoardPosition);

#[derive(Clone)]
pub struct ReversiState {
    /// The underlying 2d array of board pieces.
    board: Board,

    /// The player whose turn it currently is.
    current_player_turn: PlayerColor,
}

impl ReversiState {
    fn new() -> Self {
        let board: Board = [[None; BOARD_SIZE]; BOARD_SIZE];

        ReversiState {
            board,
            current_player_turn: PlayerColor::Black,
        }
    }

    fn transform_coords(position: BoardPosition) -> (usize, usize) {
        (position.col, BOARD_SIZE - position.row - 1)
    }

    /// Given an (x,y) coord within range of the board, return the ReversiPiece
    /// present on that spot, or None if the position is empty.
    /// Note: (0,0) is the bottom-left position.
    pub(super) fn get_piece(&self, position: BoardPosition) -> Option<ReversiPiece> {
        let (col_p, row_p) = ReversiState::transform_coords(position);

        self.board[row_p][col_p]
    }

    /// Set the piece at the coordinates to the given piece.
    fn set_piece(&mut self, position: BoardPosition, piece: Option<ReversiPiece>) {
        let (col_p, row_p) = ReversiState::transform_coords(position);

        self.board[row_p][col_p] = piece;
    }

    fn flip_piece(&mut self, position: BoardPosition) {
        let before_flip = self.get_piece(position);
        let flipped = match before_flip {
            Some(ReversiPiece::White) => Some(ReversiPiece::Black),
            Some(ReversiPiece::Black) => Some(ReversiPiece::White),
            None => panic!("attempted to flip a position that is empty."),
        };

        self.set_piece(position, flipped);
    }

    /// Since the human-friendly output is always the same size,
    /// might as well pre-compute it so we can reserve the space ahead of time.
    /// (A test exists to confirm this is accurate.)
    const fn friendly_print_size() -> usize {
        189
    }

    fn within_board_bounds(position: BoardPosition) -> bool {
        position.col < BOARD_SIZE && position.row < BOARD_SIZE
    }

    fn traverse_from(
        &self,
        origin: BoardPosition,
        direction: Directions,
    ) -> impl Iterator<Item = PiecePos> + '_ {
        util::BoardDirectionIter::new(&self, origin, direction)
    }

    /// Given a position of a piece on the board,
    /// find its sibling piece in a given direction.
    ///
    /// A sibling piece is defined as a piece of the same color that,
    /// combined with the current piece, traps enemies in the given direction.
    ///
    /// Examples:
    ///     In the below case, the pieces at 'a' and 'b'
    ///     are siblings, since together they surrouned the 3 enemy pieces.
    ///         X O O O X
    ///         a       b
    ///
    ///     In the below case, the pieces at 'a' and 'b'
    ///     are NOT siblings, since there is a gap (empty space) at 'x' preventing them
    ///     from trapping the other pieces.
    ///         X O _ O X
    ///         a   x   b
    ///
    /// This function only checks for a sibling in the given direction.
    ///
    /// If a sibling is found, it returns the BoardPosition of that sibling.
    /// Otherwise, it gives None.
    fn find_sibling_piece_pos(
        &self,
        origin: BoardPosition,
        origin_color: ReversiPiece,
        directions: Directions,
    ) -> Option<BoardPosition> {
        // 'Same' for both directions means we are not checking anything
        if directions.col_dir == SAME && directions.row_dir == SAME {
            return None;
        }

        // Distance: For every given direction, check every distance away in that direction for the terminating position.
        //      We can stop when we exceed the board range, or find another piece of our own color, as those are not valid flip directions.
        //      A legal terminating point is one where we encounter only opponent pieces, ending with an empty position.
        for col_dist in 1..BOARD_SIZE as i32 {
            let col_pos = (origin.col as i32) + (col_dist * directions.col_dir);

            if col_pos < 0 || col_pos >= BOARD_SIZE as i32 {
                // We reached the boundaries without encountering a sibling piece.
                return None;
            }

            for row_dist in 1..BOARD_SIZE as i32 {
                let row_pos = (origin.row as i32) + (row_dist * directions.row_dir);

                if row_pos < 0 || row_pos >= BOARD_SIZE as i32 {
                    // We reached the boundaries without encountering a sibling piece.
                    return None;
                }

                // Invariant: (col_pos, row_pos) must now be a position in range of the board.
                let piece = self.get_piece(BoardPosition::new(col_pos as usize, row_pos as usize));

                if piece.is_none() {
                    // This direction is not valid, since it did not end in a piece of our color.
                    return None;
                } else if piece.unwrap() == util::opponent(origin_color) {
                    // We are still in the 'opponent' segment, so keep going.
                    continue;
                } else if piece.unwrap() == origin_color {
                    // We've found another piece of our own color.
                    // But it is only a sibling piece if it traps an enemy piece (must be >1 piece away).
                    if row_dist > 1 || col_dist > 1 {
                        return Some(BoardPosition {
                            col: col_pos as usize,
                            row: row_pos as usize,
                        });
                    } else {
                        return None;
                    }
                }
            }
        }

        None
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

        result.reserve(ReversiState::friendly_print_size());

        result.push('\n');

        for row in (0..BOARD_SIZE).rev() {
            result.push_str("| ");

            for col in 0..BOARD_SIZE {
                let piece = self.get_piece(BoardPosition::new(col, row));

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
        for _ in 0..BOARD_SIZE {
            result.push_str("--");
        }

        result.push('\n');
        result.push_str("  ");
        for col in 0..BOARD_SIZE {
            result.push_str(&format!("{} ", col));
        }

        result
    }

    /// Returns the possible moves the given player can make for the current state.
    fn legal_moves(&self, player: PlayerColor) -> Vec<Self::Move> {
        Vec::new()
    }

    /// Apply the given move (or 'action') to this state, mutating this state
    /// and advancing it to the resulting state.
    /// In the game of Reversi, this places the piece at the given position,
    /// and flips all opponent pieces in any direction that terminates with another one of our pieces.
    ///
    /// Example:
    ///    O X X X *
    ///          X X
    ///        O   X
    ///            X
    ///             
    /// Placing a white piece ('O') at the position marked with the * will result in the following state:
    ///
    ///    O O O O O
    ///          O X
    ///        O   X
    ///            X
    fn apply_move(&mut self, action: Self::Move) {
        if !ReversiState::within_board_bounds(action.position) {
            panic!("Provided position exceeds bounds: {:?}", action.position);
        }

        if self.get_piece(action.position).is_some() {
            panic!("Cannot place a piece at a location that already contains a piece. Position: ({},{})");
        }

        self.set_piece(action.position, Some(action.piece));

        let all_directions = [POSITIVE, NEGATIVE, SAME];

        // Direction: For col and row, we check all directions for which pieces to flip.
        //      For col, we can check all cols to the left (direction -1), right (direction 1), or the current col (direction 0).
        //      For row, we can check all rows below us (direction -1), above us (direction 1), or the current row (direction 0).
        //      Checking all directions, including diagonals, means checking all combinations of row/col directions together (except 0,0).
        for col_dir in all_directions.iter() {
            for row_dir in all_directions.iter() {
                let directions = Directions {
                    col_dir: *col_dir,
                    row_dir: *row_dir,
                };
                let origin = action.position;
                let sibling = self.find_sibling_piece_pos(origin, action.piece, directions);

                if sibling.is_some() {
                    // have an iterator for getting pieces in a direction directions
                    // like: for piece in self.traverse_from(origin: (col, row), direction: (dir, dir), distance: usize) { /* flip */}
                }
            }
        }
    }

    /// Returns the current player whose turn it currently is.
    fn current_player_turn(&self) -> PlayerColor {
        self.current_player_turn
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Board, BoardPosition, GameState, PlayerColor, ReversiMove, ReversiPiece, ReversiState,
        BOARD_SIZE,
    };

    fn pos(col: usize, row: usize) -> BoardPosition {
        BoardPosition::new(col, row)
    }

    #[test]
    fn it_works() {
        let mut state = ReversiState::new();

        state.set_piece(pos(2, 3), Some(ReversiPiece::Black));
        let stringified = state.human_friendly();

        println!("{}", stringified);

        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn human_friendly_reserves_correct_size() {
        let state = ReversiState::new();

        let stringified = state.human_friendly();

        assert_eq!(ReversiState::friendly_print_size(), stringified.len());
    }

    #[test]
    fn state_can_set_and_get_piece() {
        let mut state = ReversiState::new();

        let piece_before = state.get_piece(pos(2, 3));

        state.set_piece(pos(2, 3), Some(ReversiPiece::White));

        let piece_after = state.get_piece(pos(2, 3));

        assert_eq!(None, piece_before);
        assert_eq!(Some(ReversiPiece::White), piece_after);
    }

    #[test]
    fn flip_piece_flips_piece() {
        let mut state = ReversiState::new();
        state.set_piece(pos(2, 3), Some(ReversiPiece::White));

        state.flip_piece(pos(2, 3));

        let flipped_piece = state.get_piece(pos(2, 3));

        assert_eq!(Some(ReversiPiece::Black), flipped_piece);
    }

    #[test]
    #[should_panic]
    fn flip_piece_panics_when_empty() {
        let mut state = ReversiState::new();

        // ensure the position is empty
        state.set_piece(pos(2, 3), None);

        // this should panic.
        state.flip_piece(pos(2, 3));
    }

    #[test]
    fn apply_move_flips_pieces_simple() {
        let mut state = ReversiState::new();

        // We have two pieces next to each other, like this:
        // O X
        state.set_piece(pos(2, 2), Some(ReversiPiece::White));
        state.set_piece(pos(3, 2), Some(ReversiPiece::Black));

        // We place a white piece at the asterisk:
        // O X *
        let action = ReversiMove {
            piece: ReversiPiece::White,
            position: pos(4, 2),
        };

        state.apply_move(action);

        // All three pieces should now be white.
        assert_eq!(ReversiPiece::White, state.get_piece(pos(2, 2)).unwrap());
        assert_eq!(ReversiPiece::White, state.get_piece(pos(3, 2)).unwrap());
        assert_eq!(ReversiPiece::White, state.get_piece(pos(4, 2)).unwrap());
    }

    #[test]
    fn apply_move_flips_pieces_complex() {
        let mut state = ReversiState::new();

        // We have this arrangemnt of pieces on the board:
        //       X
        //     O
        //   O
        // * O O O X
        state.set_piece(pos(2, 2), Some(ReversiPiece::White));
        state.set_piece(pos(3, 2), Some(ReversiPiece::White));
        state.set_piece(pos(4, 2), Some(ReversiPiece::White));
        state.set_piece(pos(5, 2), Some(ReversiPiece::Black));

        state.set_piece(pos(2, 3), Some(ReversiPiece::White));
        state.set_piece(pos(3, 4), Some(ReversiPiece::White));
        state.set_piece(pos(4, 5), Some(ReversiPiece::Black));

        // We place a black piece at the asterisk:
        let action = ReversiMove {
            piece: ReversiPiece::Black,
            position: pos(1, 2),
        };

        state.apply_move(action);

        // All pieces should now be black.
        assert_eq!(ReversiPiece::Black, state.get_piece(pos(1, 2)).unwrap());
        assert_eq!(ReversiPiece::Black, state.get_piece(pos(2, 2)).unwrap());
        assert_eq!(ReversiPiece::Black, state.get_piece(pos(3, 2)).unwrap());
        assert_eq!(ReversiPiece::Black, state.get_piece(pos(4, 2)).unwrap());
        assert_eq!(ReversiPiece::Black, state.get_piece(pos(5, 2)).unwrap());
        assert_eq!(ReversiPiece::Black, state.get_piece(pos(2, 3)).unwrap());
        assert_eq!(ReversiPiece::Black, state.get_piece(pos(3, 4)).unwrap());
        assert_eq!(ReversiPiece::Black, state.get_piece(pos(4, 5)).unwrap());
    }
}
