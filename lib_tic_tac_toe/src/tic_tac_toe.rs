use crate::tic_tac_toe_gamestate::TicTacToeState;
use lib_boardgame::{Game, GameAgent, GameResult, GameState, PlayerColor};
use std::borrow::Borrow;

pub struct TicTacToe {
    white_agent: Box<dyn GameAgent<TicTacToeState>>,
    black_agent: Box<dyn GameAgent<TicTacToeState>>,
    game_state: TicTacToeState,
}

impl TicTacToe {
    pub fn new(
        white_agent: Box<dyn GameAgent<TicTacToeState>>,
        black_agent: Box<dyn GameAgent<TicTacToeState>>,
    ) -> Self {
        Self {
            white_agent,
            black_agent,
            game_state: TicTacToeState::initial_state(),
        }
    }
}

impl Game for TicTacToe {
    type State = TicTacToeState;

    fn white_agent(&self) -> &dyn GameAgent<TicTacToeState> {
        self.white_agent.borrow()
    }

    fn black_agent(&self) -> &dyn GameAgent<TicTacToeState> {
        self.black_agent.borrow()
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
        if self.is_game_over() {
            return match self.game_state().get_winner() {
                Some(PlayerColor::Black) => Some(GameResult::BlackWins),
                Some(PlayerColor::White) => Some(GameResult::WhiteWins),
                None => Some(GameResult::Tie),
            };
        }

        None
    }
}
