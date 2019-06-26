use crate::*;

#[derive(Clone, Default)]
pub struct TestGameState {
    child_states: Vec<TestActionResult>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TestMove;

#[derive(Clone, Default)]
pub struct TestActionResult {
    action: TestMove,
    result_state: TestGameState,
}

impl GameMove for TestMove {}

impl GameState for TestGameState {
    type Move = TestMove;

    /// Returns a human-friendly string for representing the state.
    fn human_friendly(&self) -> String {
        String::default()
    }

    /// Gives the implementation a chance to initialize the starting state of a game
    /// before gameplay begins.
    fn initialize_board(&mut self) {
        // do nothing
    }

    /// Returns a fresh, ready-to-play game state for this game.
    fn initial_state() -> Self {
        unimplemented!()
    }

    /// Returns the possible moves the given player can make for the current state.
    fn legal_moves(&self, _player: PlayerColor) -> Vec<Self::Move> {
        unimplemented!()
    }

    /// Apply the given move (or 'action') to this state, mutating this state
    /// and advancing it to the resulting state.
    fn apply_move(&mut self, _action: Self::Move) {
        unimplemented!()
    }

    /// Returns the current player whose turn it currently is.
    fn current_player_turn(&self) -> PlayerColor {
        unimplemented!()
    }

    /// Returns the score of the given player in this state.
    fn player_score(&self, _player: PlayerColor) -> usize {
        unimplemented!()
    }

    /// Skip the current player's turn without taking any action.
    /// Advances to the next player's turn.
    fn skip_turn(&mut self) {
        unimplemented!()
    }

    /// True if the game is over (i.e. neither player can take any further action).
    fn is_game_over(&self) -> bool {
        unimplemented!()
    }
}
