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
pub trait GameMove {}

/// Describes a complete state of some Game,
/// such as the board position, the current player's turn,
/// or any other relevant info.
pub trait GameState {
    type Move: GameMove;

    /// Returns a human-friendly string for representing the state.
    fn human_friendly(&self) -> String;

    /// Returns the possible moves the given player can make for the current state.
    fn legal_moves(&self, player: PlayerColor) -> Vec<Self::Move>;
}

pub trait Game {
    type State: GameState;
    type WhiteAgent: GameAgent<Self::State>;
    type BlackAgent: GameAgent<Self::State>;

    /// Returns the player whose turn it is.
    fn whose_turn(&self) -> PlayerColor;

    fn white_agent(&self) -> &Self::WhiteAgent;
    fn black_agent(&self) -> &Self::BlackAgent;

    /// The game's current state.
    fn game_state(&self) -> &Self::State;

    /// True if the the game has ended, either due to a forced win,
    /// draw, or forfeit.
    fn is_game_over(&self) -> bool;

    /// The GameResult, or None if the game is not yet over.
    fn game_result(&self) -> Option<GameResult>;

    /// Invokes the current player agent to pick a move,
    /// then updates the game state and gives control to the next player.
    fn player_take_turn(&mut self);

    fn play_to_end(&mut self) -> GameResult {
        while !self.is_game_over() {
            match self.whose_turn() {
                PlayerColor::Black => self.player_take_turn(),
                PlayerColor::White => self.player_take_turn(),
            }
        }

        GameResult::BlackWins
    }
}

pub trait GameAgent<TState: GameState> {
    fn pick_move(state: &TState);
}

mod tests {
    use super::*;
}
