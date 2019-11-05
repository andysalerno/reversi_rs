use crate::reversi_gamestate::ReversiState;
use lib_boardgame::{Game, GameAgent, GameResult, GameState};
use std::borrow::Borrow;

pub struct Reversi {
    white_agent: Box<dyn GameAgent<ReversiState>>,
    black_agent: Box<dyn GameAgent<ReversiState>>,
    game_state: ReversiState,
}

impl Reversi {
    pub fn new(
        white_agent: Box<dyn GameAgent<ReversiState>>,
        black_agent: Box<dyn GameAgent<ReversiState>>,
    ) -> Self {
        Reversi {
            white_agent,
            black_agent,
            game_state: ReversiState::new(),
        }
    }
}

impl Game for Reversi {
    type State = ReversiState;

    fn white_agent(&self) -> &dyn GameAgent<ReversiState> {
        self.white_agent.borrow()
    }
    fn black_agent(&self) -> &dyn GameAgent<ReversiState> {
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
        let gamestate = self.game_state();

        gamestate.is_game_over()
    }

    fn game_result(&self) -> Option<GameResult> {
        self.game_state().game_result()
    }
}
