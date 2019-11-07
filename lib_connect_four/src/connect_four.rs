use lib_boardgame::{GameResult, GameState, PlayerColor};
use std::fmt::Display;

const GAME_WIDTH: usize = 7;
const GAME_HEIGHT: usize = GAME_WIDTH - 1;

fn is_in_range(col: usize, height: usize) -> bool {
    col < GAME_WIDTH && height < GAME_HEIGHT
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ConnectFourPiece {
    Black,
    Red,
    Empty,
}

impl Display for ConnectFourPiece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let r = match self {
            ConnectFourPiece::Black => "X",
            ConnectFourPiece::Red => "O",
            ConnectFourPiece::Empty => " ",
        };

        write!(f, "{}", r)
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct ConnectFourAction {
    /// The col index where the piece will be dropped,
    /// where 0 is the leftmost col and GAME_SIZE-1 is the rightmost.
    col: usize,
}

impl ConnectFourAction {
    fn new(col: usize) -> Self {
        Self { col }
    }
}

impl lib_boardgame::GameAction for ConnectFourAction {
    fn is_forced_pass(self) -> bool {
        // No such thing in this game
        false
    }
}

impl std::str::FromStr for ConnectFourAction {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let col: usize = s.trim().parse()?;

        Ok(ConnectFourAction::new(col))
    }
}

impl Display for ConnectFourAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "col: {}", self.col)
    }
}

#[derive(Clone, Debug)]
pub struct ConnectFourState {
    player_turn: PlayerColor,
    board: [[ConnectFourPiece; GAME_WIDTH]; GAME_HEIGHT],
    col_cur_height: [usize; GAME_WIDTH],

    legal_moves: Vec<ConnectFourAction>,
    game_result: Option<GameResult>,

    /// Count of pieces on the board.
    piece_count: usize,
}

impl ConnectFourState {
    pub fn new() -> Self {
        Self {
            player_turn: PlayerColor::Black,
            col_cur_height: [0; GAME_WIDTH],
            board: [[ConnectFourPiece::Empty; GAME_WIDTH]; GAME_HEIGHT],

            legal_moves: Default::default(),
            game_result: None,
            piece_count: 0,
        }
    }

    /// Refresh the cached value of legal moves available from this state.
    fn update_legal_moves(&mut self) {
        let legal = (0..GAME_WIDTH)
            .filter(|&i| !self.is_col_full(i))
            .map(|i| ConnectFourAction::new(i))
            .collect::<Vec<_>>();

        std::mem::replace(&mut self.legal_moves, legal);
    }

    /// Returns the piece at the given position.
    fn piece_at(&self, col: usize, height: usize) -> ConnectFourPiece {
        self.board[height][col]
    }

    /// Sets the piece at the given location.
    /// Does NOT refresh state-based values, such as piece_count or game_result.
    fn set_piece(&mut self, col: usize, height: usize, piece: ConnectFourPiece) {
        self.board[height][col] = piece;
    }

    /// The height of the given column. E.g., an empty column has height 0.
    fn col_height(&self, col: usize) -> usize {
        self.col_cur_height[col]
    }

    /// True if the column is full (has no room left for other pieces).
    fn is_col_full(&self, col: usize) -> bool {
        self.col_height(col) >= GAME_HEIGHT
    }

    /// Increment the cached column height. Internal use only.
    fn increment_col(&mut self, col: usize) {
        self.col_cur_height[col] += 1;
    }

    fn increment_piece_count(&mut self) {
        self.piece_count += 1;
    }

    fn update_end_game_result(&mut self) {
        if self.piece_count >= GAME_HEIGHT * GAME_WIDTH {
            self.game_result = Some(GameResult::Tie);
        }
    }

    /// "Drop" a piece at the given column. The piece "falls" from the top
    /// and stops at the first position that is above another piece.
    pub fn drop_piece(&mut self, col: usize, piece: ConnectFourPiece) {
        let piece_height = self.col_height(col);

        if piece_height >= GAME_HEIGHT {
            panic!(
                "can't legally drop a piece in col {}, \
                 which already has height {} and has no more room.",
                col, piece_height
            );
        }

        self.set_piece(col, piece_height, piece);
        self.increment_col(col);

        let pos = Position {
            x: col,
            y: piece_height,
        };

        if self.is_pos_four_in_a_row(pos) {
            self.game_result = match piece {
                ConnectFourPiece::Black => Some(GameResult::BlackWins),
                ConnectFourPiece::Red => Some(GameResult::WhiteWins),
                _ => {
                    panic!("Can only consider a game won if a black or red piece was lost dropped")
                }
            };
        }

        self.update_legal_moves();
        self.increment_piece_count();
        self.update_end_game_result();
    }

    fn is_pos_four_in_a_row(&self, pos: Position) -> bool {
        if self.piece_at(pos.x, pos.y) == ConnectFourPiece::Empty {
            return false;
        }

        // Left, right
        let col_dirs = &[-1, 0, 1];

        // Down, up
        let height_dirs = &[-1, 0, 1];

        for &col_dir in col_dirs {
            for &height_dir in height_dirs {
                if col_dir == 0 && height_dir == 0 {
                    // No net movement, so skip this one
                    continue;
                }

                let origin_color = self.piece_at(pos.x, pos.y);

                let mut dir_run_len = 1;

                // Traverse the direction forwards and backwards
                for s in &[-1, 1] {
                    for i in 1.. {
                        let col: i32 = pos.x as i32 + (s * i * col_dir);
                        let height: i32 = pos.y as i32 + (s * i * height_dir);

                        if col < 0 || height < 0 || !is_in_range(col as usize, height as usize) {
                            break;
                        }

                        if self.piece_at(col as usize, height as usize) == origin_color {
                            dir_run_len += 1;
                        } else {
                            // The single-color run has ended, so stop traversing
                            break;
                        }
                    }
                }

                if dir_run_len >= 4 {
                    return true;
                }
            }
        }

        false
    }
}

impl Display for ConnectFourState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut result = String::new();

        for height in (0..GAME_HEIGHT).rev() {
            result.push('|');

            for col in 0..GAME_WIDTH {
                let piece = self.piece_at(col, height);
                result.push_str(&format!("{}", piece));
                result.push('|');
            }

            result.push('\n');
        }

        result.push(' ');
        for col in 0..GAME_WIDTH {
            result.push_str(&format!("{} ", col.to_string()));
        }

        write!(f, "{}", result)
    }
}

impl GameState for ConnectFourState {
    type Action = ConnectFourAction;

    fn human_friendly(&self) -> String {
        format!("{}", self)
    }

    fn initialize_board(&mut self) {
        for height_slice in &mut self.board {
            for pos in height_slice {
                *pos = ConnectFourPiece::Empty;
            }
        }
    }

    fn initial_state() -> Self {
        let mut state = Self::new();
        state.update_legal_moves();

        state
    }

    fn legal_moves(&self, _player: PlayerColor) -> &[Self::Action] {
        &self.legal_moves
    }

    fn apply_move(&mut self, action: Self::Action) {
        let col = action.col;

        let piece = match self.current_player_turn() {
            PlayerColor::Black => ConnectFourPiece::Black,
            PlayerColor::White => ConnectFourPiece::Red,
        };

        self.drop_piece(col, piece);

        self.player_turn = self.player_turn.opponent();
    }

    fn current_player_turn(&self) -> PlayerColor {
        self.player_turn
    }

    fn player_score(&self, _player: PlayerColor) -> usize {
        unimplemented!()
    }

    fn skip_turn(&mut self) {
        unimplemented!()
    }

    fn is_game_over(&self) -> bool {
        self.game_result.is_some()
    }

    fn game_result(&self) -> Option<GameResult> {
        self.game_result
    }
}

struct Position {
    x: usize,
    y: usize,
}
