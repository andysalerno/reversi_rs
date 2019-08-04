use crate::{TicTacToePiece, BOARD_SIZE};
use lib_boardgame::{GameMove, GameMoveFromStr, GameState, PlayerColor};

type Board = [[Option<TicTacToePiece>; BOARD_SIZE]; BOARD_SIZE];

#[derive(Clone, Copy, Debug)]
pub struct TicTacToeState {
    board: Board,
    x_piece_count: usize,
    o_piece_count: usize,
    current_player_turn: PlayerColor,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TicTacToeAction(pub BoardPosition);

impl GameMove for TicTacToeAction {}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct BoardPosition {
    col: usize,
    row: usize,
}

impl BoardPosition {
    pub fn new(col: usize, row: usize) -> Self {
        BoardPosition { col, row }
    }
}

impl std::str::FromStr for TicTacToeAction {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let nums: Vec<_> = s.split(',').map(|x| x.trim()).collect();

        if nums.len() != 2 {
            println!("Invalid input: {} -- expected format: col,row", s);
            return Err(());
        }

        let col = nums[0].parse::<usize>();
        let row = nums[1].parse::<usize>();

        if let (Ok(col), Ok(row)) = (col, row) {
            let position = BoardPosition::new(col, row);

            if col > BOARD_SIZE || row >= BOARD_SIZE {
                println!(
                    "Position out of bounds of board. Input: {:?}, actual board size: {}",
                    position, BOARD_SIZE
                );

                return Err(());
            } else {
                let action = TicTacToeAction(position);
                return Ok(action);
            }
        } else {
            println!("Didn't recognize input as a board position: {}", s);
            return Err(());
        }
    }
}

impl GameMoveFromStr for TicTacToeAction {
    fn from_str(s: &str, _player_color: PlayerColor) -> Result<Self, Self::Err> {
        let action: TicTacToeAction = std::str::FromStr::from_str(s)?;

        Ok(action)
    }
}

impl TicTacToeState {
    pub fn new() -> Self {
        let board: Board = [[None; BOARD_SIZE]; BOARD_SIZE];

        Self {
            board,
            x_piece_count: 0,
            o_piece_count: 0,
            current_player_turn: PlayerColor::Black,
        }
    }

    fn transform_coords(position: BoardPosition) -> (usize, usize) {
        (position.col, BOARD_SIZE - position.row - 1)
    }

    pub(super) fn get_piece(&self, position: BoardPosition) -> Option<TicTacToePiece> {
        let (col_p, row_p) = Self::transform_coords(position);

        self.board[row_p][col_p]
    }

    /// Set the piece at the coordinates to the given piece.
    fn set_piece(&mut self, position: BoardPosition, piece: Option<TicTacToePiece>) {
        let (col_p, row_p) = Self::transform_coords(position);

        let existing = self.board[row_p][col_p];

        match existing {
            Some(TicTacToePiece::X) => self.x_piece_count -= 1,
            Some(TicTacToePiece::O) => self.o_piece_count -= 1,
            _ => {}
        };

        match piece {
            Some(TicTacToePiece::X) => self.x_piece_count += 1,
            Some(TicTacToePiece::O) => self.o_piece_count += 1,
            _ => {}
        };

        self.board[row_p][col_p] = piece;
    }

    /// Since the human-friendly output is always the same size,
    /// might as well pre-compute it so we can reserve the space ahead of time.
    /// (A test exists to confirm this is accurate.)
    const fn friendly_print_size() -> usize {
        18
    }

    fn within_board_bounds(position: BoardPosition) -> bool {
        position.col < BOARD_SIZE && position.row < BOARD_SIZE
    }

    pub fn get_winner(&self) -> Option<PlayerColor> {
        // Does there exist any row, column, or diagonal
        // which belongs entirely to one player?
        // If so, that player has won the game.

        // Rows
        for y in 0..BOARD_SIZE {
            let first_piece = self.get_piece(BoardPosition::new(0, y));
            if first_piece.is_none() {
                // This row is a bust, so move on to the next.
                continue;
            }

            for x in 1..BOARD_SIZE {
                let piece = self.get_piece(BoardPosition::new(x, y));

                if piece != first_piece {
                    // This row is a bust, try the next one.
                    break;
                }

                if x == (BOARD_SIZE - 1) {
                    // We made it to the final position without failing,
                    // so we must have found a full diagonal populated by one player's piece.
                    // Therefore, the game is won.
                    return Some(first_piece.unwrap().player_color());
                }
            }
        }

        // Columns
        for x in 0..BOARD_SIZE {
            let first_piece = self.get_piece(BoardPosition::new(x, 0));
            if first_piece.is_none() {
                // This row is a bust, so move on to the next.
                continue;
            }

            for y in 1..BOARD_SIZE {
                let piece = self.get_piece(BoardPosition::new(x, y));

                if piece != first_piece {
                    // This row is a bust, try the next one.
                    break;
                }

                if y == (BOARD_SIZE - 1) {
                    // We made it to the final position without failing,
                    // so we must have found a full diagonal populated by one player's piece.
                    // Therefore, the game is won.
                    return Some(first_piece.unwrap().player_color());
                }
            }
        }

        // Diagonals
        // Top-left to bottom-right
        {
            let top_left_first_piece = self.get_piece(BoardPosition::new(0, BOARD_SIZE - 1));
            if top_left_first_piece.is_some() {
                for xy in 1..BOARD_SIZE {
                    let piece = self.get_piece(BoardPosition::new(xy, BOARD_SIZE - xy - 1));

                    if piece != top_left_first_piece {
                        break;
                    }

                    if xy == BOARD_SIZE - 1 {
                        // We made it to the final position without failing,
                        // so we must have found a full diagonal populated by one player's piece.
                        // Therefore, the game is won.
                        return Some(top_left_first_piece.unwrap().player_color());
                    }
                }
            }
        }

        // Bottom-left to top-right
        {
            let bottom_left_first_piece = self.get_piece(BoardPosition::new(0, 0));
            if bottom_left_first_piece.is_some() {
                for xy in 1..BOARD_SIZE {
                    let piece = self.get_piece(BoardPosition::new(xy, xy));

                    if piece != bottom_left_first_piece {
                        break;
                    }

                    if xy == BOARD_SIZE - 1 {
                        // We made it to the final position without failing,
                        // so we must have found a full diagonal populated by one player's piece.
                        // Therefore, the game is won.
                        return Some(bottom_left_first_piece.unwrap().player_color());
                    }
                }
            }
        }

        None
    }
}

impl GameState for TicTacToeState {
    type Move = TicTacToeAction;

    /// Returns a human-friendly string for representing the state.
    fn human_friendly(&self) -> String {
        let mut result = String::with_capacity(Self::friendly_print_size());

        for y in (0..BOARD_SIZE).rev() {
            for x in 0..BOARD_SIZE {
                let position = BoardPosition::new(x, y);
                let symbol = match self.get_piece(position) {
                    None => '_',
                    Some(TicTacToePiece::X) => 'X',
                    Some(TicTacToePiece::O) => 'O',
                };

                result.push(symbol);

                if x != BOARD_SIZE - 1 {
                    result.push('|');
                }
            }

            result.push('\n');
        }

        result
    }

    /// Gives the implementation a chance to initialize the starting state of a game
    /// before gameplay begins.
    fn initialize_board(&mut self) {
        for y in 0..self.board.len() {
            for x in 0..self.board[0].len() {
                self.board[y][x] = None;
            }
        }
    }

    /// Returns a fresh, ready-to-play game state for this game.
    fn initial_state() -> Self {
        let mut uninitialized = Self::new();
        uninitialized.initialize_board();

        uninitialized
    }

    /// Returns the possible moves the given player can make for the current state.
    /// In TicTacToe, any empty spot is a legal position for either player.
    fn legal_moves(&self, _player: PlayerColor) -> Vec<Self::Move> {
        let mut actions = Vec::with_capacity(BOARD_SIZE * BOARD_SIZE);

        for y in 0..BOARD_SIZE {
            for x in 0..BOARD_SIZE {
                let position = BoardPosition::new(x, y);
                match self.get_piece(position) {
                    Some(_) => {}
                    None => actions.push(TicTacToeAction(position)),
                }
            }
        }

        actions
    }

    /// Apply the given move (or 'action') to this state, mutating this state
    /// and advancing it to the resulting state.
    fn apply_move(&mut self, action: TicTacToeAction) {
        if !Self::within_board_bounds(action.0) {
            panic!("The provided action is illegal because the board position is out of bounds.");
        }

        if self.get_piece(action.0).is_some() {
            panic!("Cannot place piece at position {:?} (another piece exists there", action.0);
        }

        let piece = match self.current_player_turn() {
            PlayerColor::Black => TicTacToePiece::X,
            PlayerColor::White => TicTacToePiece::O,
        };
        self.set_piece(action.0, Some(piece));

        self.current_player_turn = self.current_player_turn.opponent();
    }

    /// Returns the current player whose turn it currently is.
    fn current_player_turn(&self) -> PlayerColor {
        self.current_player_turn
    }

    /// Returns the score of the given player in this state.
    fn player_score(&self, player: PlayerColor) -> usize {
        const WIN: usize = 1;
        const NOT_WIN: usize = 0;

        match self.get_winner() {
            Some(PlayerColor::White) if player == PlayerColor::White => WIN,
            Some(PlayerColor::Black) if player == PlayerColor::Black => WIN,
            _ => NOT_WIN 
        }
    }

    /// Skip the current player's turn without taking any action.
    /// Advances to the next player's turn.
    fn skip_turn(&mut self) {
        panic!("Skipping turns is not a valid operation in TicTacToe.");
    }

    /// True if the game is over (i.e. the win condition has been met, or neither player can take any further action).
    fn is_game_over(&self) -> bool {
        self.get_winner().is_some()
            || self.x_piece_count + self.o_piece_count == (BOARD_SIZE * BOARD_SIZE)
    }
}
