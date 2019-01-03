pub enum PlayerColor {
    Black,
    White,
}

pub enum GameResult {
    Tie,
    WhiteWins,
    BlackWins,
}

/// Describes a move a player can make in a game.
/// I.e., in Reversi, a move could be at position (3,7).
pub trait GameMove: Copy {}

/// Describes a complete state of some Game,
/// such as the board position, the current player's turn,
/// or any other relevant info.
pub trait GameState: Clone {
    type Move: GameMove;

    /// Returns a human-friendly string for representing the state.
    fn human_friendly(&self) -> String;

    /// Returns the possible moves the given player can make for the current state.
    fn legal_moves(&self, player: PlayerColor) -> Vec<Self::Move>;

    /// Apply the given move (or 'action') to this state, mutating this state
    /// and advancing it to the resulting state.
    fn apply_move(&mut self, action: Self::Move);

    /// Given a legal move (or 'action'), return the resulting state of applying the action
    /// to this state (does not mutate this state).
    fn next_state(&self, action: Self::Move) -> Self {
        let mut cloned = self.clone();
        cloned.apply_move(action);

        cloned
    }
}

pub trait Game<WhiteAgent, BlackAgent>
where
    WhiteAgent: GameAgent<Self::State>,
    BlackAgent: GameAgent<Self::State>,
{
    type State: GameState;

    /// Returns the player whose turn it is.
    fn whose_turn(&self) -> PlayerColor;

    fn white_agent(&self) -> &WhiteAgent;
    fn black_agent(&self) -> &BlackAgent;

    /// The game's current state.
    fn game_state(&self) -> &Self::State;

    /// The game's current state.
    fn game_state_mut(&self) -> &mut Self::State;

    /// True if the the game has ended, either due to a forced win,
    /// draw, or forfeit.
    fn is_game_over(&self) -> bool;

    /// The GameResult, or None if the game is not yet over.
    fn game_result(&self) -> Option<GameResult>;

    /// Invokes the current player agent to pick a move,
    /// then updates the game state and gives control to the next player.
    fn player_take_turn(&mut self, player: PlayerColor) {
        let state = self.game_state_mut();

        let picked_move = match player {
            PlayerColor::Black => self.black_agent().pick_move(state),
            PlayerColor::White => self.white_agent().pick_move(state),
        };

        state.apply_move(picked_move);
    }

    /// Applies each player's turn one at a time until the game is over,
    /// and returns the game result.
    fn play_to_end(&mut self) -> GameResult {
        while !self.is_game_over() {
            let cur_player_color = self.whose_turn();
            self.player_take_turn(cur_player_color);
        }

        GameResult::BlackWins
    }
}

/// A trait representing the functionality of a GameAgent.
/// Specifically, given a GameState, a GameAgent must be able to decide a GameMove.
pub trait GameAgent<TState: GameState> {
    fn pick_move(&self, state: &TState) -> TState::Move;
}

mod tests {
    use super::*;
}
