use lib_boardgame::{Game, GameAgent, GameResult, GameState, PlayerColor};
use std::fmt::Display;

const GAME_WIDTH: usize = 7;
const GAME_HEIGHT: usize = GAME_WIDTH - 1;

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
    /// The row index where the piece will be dropped,
    /// where 0 is the leftmost row and GAME_SIZE-1 is the rightmost.
    row: usize,
}

impl ConnectFourAction {
    fn new(row: usize) -> Self {
        Self { row }
    }
}

impl lib_boardgame::GameMove for ConnectFourAction {
    fn is_forced_pass(self) -> bool {
        // No such thing in this game
        false
    }
}

impl Display for ConnectFourAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "row: {}", self.row)
    }
}

#[derive(Clone, Debug)]
pub struct ConnectFourState {
    player_turn: PlayerColor,
    board: [[ConnectFourPiece; GAME_WIDTH]; GAME_HEIGHT],
    row_cur_height: [usize; GAME_WIDTH],

    legal_moves: Vec<ConnectFourAction>,
}

impl ConnectFourState {
    pub fn new() -> Self {
        Self {
            player_turn: PlayerColor::Black,
            row_cur_height: [0; GAME_WIDTH],
            board: [[ConnectFourPiece::Empty; GAME_WIDTH]; GAME_HEIGHT],

            legal_moves: Default::default(),
        }
    }

    fn update_legal_moves(&mut self) {
        let legal = (0..GAME_WIDTH)
            .filter(|&i| !self.is_row_full(i))
            .map(|i| ConnectFourAction::new(i))
            .collect::<Vec<_>>();

        std::mem::replace(&mut self.legal_moves, legal);
    }

    fn piece_at(&self, row: usize, height: usize) -> ConnectFourPiece {
        self.board[height][row]
    }

    fn set_piece(&mut self, row: usize, height: usize, piece: ConnectFourPiece) {
        self.board[height][row] = piece;
    }

    fn row_height(&self, row: usize) -> usize {
        self.row_cur_height[row]
    }

    fn is_row_full(&self, row: usize) -> bool {
        self.row_height(row) >= GAME_HEIGHT
    }

    pub fn drop_piece(&mut self, row: usize, piece: ConnectFourPiece) {
        let piece_height = self.row_height(row);

        if piece_height >= GAME_HEIGHT {
            panic!(
                "can't legally drop a piece in row {}, \
                 which already has height {} and has no more room.",
                row, piece_height
            );
        }

        self.set_piece(row, piece_height, piece);
        self.row_cur_height[row] += 1;
    }
}

impl Display for ConnectFourState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut result = String::new();

        for row in (0..GAME_HEIGHT).rev() {
            result.push('|');

            for row_idx in 0..GAME_WIDTH {
                let piece = self.piece_at(row_idx, row);
                result.push_str(&format!("{}", piece));
                result.push('|');
            }

            result.push('\n');
        }

        result.push(' ');
        for row in 0..GAME_WIDTH {
            result.push_str(&format!("{} ", row.to_string()));
        }

        write!(f, "{}", result)
    }
}

impl GameState for ConnectFourState {
    type Move = ConnectFourAction;

    fn human_friendly(&self) -> String {
        format!("{}", self)
    }

    fn initialize_board(&mut self) {
        for row in &mut self.board {
            for row_loc in row {
                *row_loc = ConnectFourPiece::Empty;
            }
        }
    }

    fn initial_state() -> Self {
        let mut state = Self::new();
        state.update_legal_moves();

        state
    }

    fn legal_moves(&self, _player: PlayerColor) -> &[Self::Move] {
        &self.legal_moves
    }

    fn apply_move(&mut self, action: Self::Move) {
        let row = action.row;

        let piece = match self.current_player_turn() {
            PlayerColor::Black => ConnectFourPiece::Black,
            PlayerColor::White => ConnectFourPiece::Red,
        };

        self.drop_piece(row, piece);

        self.player_turn = self.player_turn.opponent();

        self.update_legal_moves();
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
        (0..GAME_WIDTH).all(|i| self.is_row_full(i))
    }
}

pub struct ConnectFour {
    white_agent: Box<dyn GameAgent<ConnectFourState>>,
    black_agent: Box<dyn GameAgent<ConnectFourState>>,
    game_state: ConnectFourState,
}

impl ConnectFour {
    pub fn new(
        white_agent: Box<dyn GameAgent<ConnectFourState>>,
        black_agent: Box<dyn GameAgent<ConnectFourState>>,
    ) -> Self {
        Self {
            white_agent,
            black_agent,
            game_state: ConnectFourState::initial_state(),
        }
    }
}

impl Game for ConnectFour {
    type State = ConnectFourState;

    fn white_agent(&self) -> &dyn GameAgent<ConnectFourState> {
        &*self.white_agent
    }

    fn black_agent(&self) -> &dyn GameAgent<ConnectFourState> {
        &*self.black_agent
    }

    /// The game's current state.
    fn game_state(&self) -> &Self::State {
        &self.game_state
    }

    /// The game's current state.
    fn game_state_mut(&mut self) -> &mut Self::State {
        &mut self.game_state
    }

    /// True if the the game has ended, either due to a forced win,
    /// draw, or forfeit.
    fn is_game_over(&self) -> bool {
        self.game_state.is_game_over()
    }

    /// The GameResult, or None if the game is not yet over.
    fn game_result(&self) -> Option<GameResult> {
        Some(GameResult::BlackWins)
    }
}
