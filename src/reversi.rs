use crate::game_primitives::{Game, GameAgent, GameMove, GameResult, GameState, PlayerColor};

#[derive(Copy, Clone)]
struct ReversiMove;
impl GameMove for ReversiMove {}

#[derive(Clone)]
struct ReversiState;

impl GameState for ReversiState {
    type Move = ReversiMove;

    /// Returns a human-friendly string for representing the state.
    fn human_friendly(&self) -> String {
        "hello".to_owned()
    }

    /// Returns the possible moves the given player can make for the current state.
    fn legal_moves(&self, player: PlayerColor) -> Vec<Self::Move> {
        Vec::new()
    }

    /// Apply the given move (or 'action') to this state, mutating this state
    /// and advancing it to the resulting state.
    fn apply_move(&mut self, action: Self::Move) {}
}

struct Reversi<WhiteAgent, BlackAgent>
where
    WhiteAgent: GameAgent<ReversiState>,
    BlackAgent: GameAgent<ReversiState>,
{
    white_agent: WhiteAgent,
    black_agent: BlackAgent,
}

impl<W, B> Game<W, B> for Reversi<W, B>
where
    W: GameAgent<ReversiState>,
    B: GameAgent<ReversiState>,
{
    type State = ReversiState;

    /// Returns the player whose turn it is.
    fn whose_turn(&self) -> PlayerColor {
        PlayerColor::White
    }

    fn white_agent(&self) -> &W {
        unimplemented!()
    }
    fn black_agent(&self) -> &B {
        unimplemented!()
    }

    /// The game's current state.
    fn game_state(&self) -> &Self::State {
        unimplemented!()
    }

    /// The game's current state.
    fn game_state_mut(&self) -> &mut Self::State {
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
