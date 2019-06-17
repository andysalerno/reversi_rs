use crate::test_impls::game_state_test_impl::TestGameState;
use crate::{Game, GameAgent, GameResult};

pub struct TestGame<WhiteAgent, BlackAgent>
where
    WhiteAgent: GameAgent<TestGameState>,
    BlackAgent: GameAgent<TestGameState>,
{
    _white_agent: WhiteAgent,
    _black_agent: BlackAgent,
    _cur_state: TestGameState,
}

impl<W, B> TestGame<W, B>
where
    W: GameAgent<TestGameState>,
    B: GameAgent<TestGameState>,
{
    pub fn new(_white_agent: W, _black_agent: B) -> Self {
        Self {
            _white_agent,
            _black_agent,
            _cur_state: TestGameState::default(),
        }
    }
}

impl<WhiteAgent, BlackAgent> Game<WhiteAgent, BlackAgent> for TestGame<WhiteAgent, BlackAgent>
where
    WhiteAgent: GameAgent<TestGameState>,
    BlackAgent: GameAgent<TestGameState>,
{
    type State = TestGameState;

    fn white_agent(&self) -> &WhiteAgent {
        unimplemented!()
    }
    fn black_agent(&self) -> &BlackAgent {
        unimplemented!()
    }

    /// The game's current state.
    fn game_state(&self) -> &Self::State {
        unimplemented!()
    }

    /// The game's current state.
    fn game_state_mut(&mut self) -> &mut Self::State {
        unimplemented!()
    }

    /// True if the the game has ended, either due to a forced win,
    /// draw, or forfeit.
    fn is_game_over(&self) -> bool {
        unimplemented!()
    }

    /// The GameResult, or None if the game is not yet over.
    fn game_result(&self) -> Option<GameResult> {
        unimplemented!()
    }
}
