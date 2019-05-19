use crate::reversi_gamestate::ReversiState;
use lib_boardgame::{Game, GameAgent, GameResult, GameState};

pub struct Reversi<WhiteAgent, BlackAgent>
where
    WhiteAgent: GameAgent<ReversiState>,
    BlackAgent: GameAgent<ReversiState>,
{
    white_agent: WhiteAgent,
    black_agent: BlackAgent,
    game_state: ReversiState,
}

impl<WhiteAgent, BlackAgent> Reversi<WhiteAgent, BlackAgent>
where
    WhiteAgent: GameAgent<ReversiState>,
    BlackAgent: GameAgent<ReversiState>,
{
    pub fn new(white_agent: WhiteAgent, black_agent: BlackAgent) -> Self {
        Reversi {
            white_agent,
            black_agent,
            game_state: ReversiState::new(),
        }
    }
}

impl<W, B> Game<W, B> for Reversi<W, B>
where
    W: GameAgent<ReversiState>,
    B: GameAgent<ReversiState>,
{
    type State = ReversiState;

    fn white_agent(&self) -> &W {
        &self.white_agent
    }
    fn black_agent(&self) -> &B {
        &self.black_agent
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
