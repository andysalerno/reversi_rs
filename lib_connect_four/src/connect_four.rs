use lib_boardgame::{Game, GameAgent, GameResult, GameState, PlayerColor};
use std::fmt::Display;

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct ConnectFourAction;

impl lib_boardgame::GameMove for ConnectFourAction {
    fn is_forced_pass(self) -> bool {
        // No such thing in this game
        false
    }
}

impl Display for ConnectFourAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unimplemented!()
    }
}

#[derive(Clone, Debug)]
pub struct ConnectFourState;

impl Display for ConnectFourState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unimplemented!()
    }
}

impl GameState for ConnectFourState {
    type Move = ConnectFourAction;

    fn human_friendly(&self) -> String {
        unimplemented!()
    }
    fn initialize_board(&mut self) {
        unimplemented!()
    }
    fn initial_state() -> Self {
        unimplemented!()
    }
    fn legal_moves(&self, player: PlayerColor) -> &[Self::Move] {
        unimplemented!()
    }
    fn apply_move(&mut self, action: Self::Move) {
        unimplemented!()
    }
    fn current_player_turn(&self) -> PlayerColor {
        unimplemented!()
    }
    fn player_score(&self, player: PlayerColor) -> usize {
        unimplemented!()
    }
    fn skip_turn(&mut self) {
        unimplemented!()
    }
    fn is_game_over(&self) -> bool {
        unimplemented!()
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
        unimplemented!()
    }
}
