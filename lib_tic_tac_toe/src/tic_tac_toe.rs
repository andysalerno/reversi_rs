use crate::tic_tac_toe_gamestate::TicTacToeState;
use lib_boardgame::{Game, GameAgent, GameResult, GameState, PlayerColor};

pub struct TicTacToe<WhiteAgent, BlackAgent>
where
    WhiteAgent: GameAgent<TicTacToeState>,
    BlackAgent: GameAgent<TicTacToeState>,
{
    white_agent: WhiteAgent,
    black_agent: BlackAgent,
    game_state: TicTacToeState,
}

impl<W, B> TicTacToe<W, B>
where
    W: GameAgent<TicTacToeState>,
    B: GameAgent<TicTacToeState>,
{
    pub fn new(white_agent: W, black_agent: B) -> Self {
        Self {
            white_agent,
            black_agent,
            game_state: TicTacToeState::initial_state(),
        }
    }
}

impl<W, B> Game<W, B> for TicTacToe<W, B>
where
    W: GameAgent<TicTacToeState>,
    B: GameAgent<TicTacToeState>,
{
    type State = TicTacToeState;

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
