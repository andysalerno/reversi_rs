mod game_runner;

use lib_printer::{out, out_impl};
use std::fmt::{Debug, Display};

pub use game_runner::{GameRunner, GeneralGameRunner};

/// An enum representing the two possible player colors for all games.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum PlayerColor {
    Black,
    White,
}

impl PlayerColor {
    /// Returns the opposing player color.
    pub fn opponent(self) -> Self {
        match self {
            PlayerColor::Black => PlayerColor::White,
            PlayerColor::White => PlayerColor::Black,
        }
    }
}

/// An enum representing the possible
/// results of a game that is played to conclusion.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum GameResult {
    Tie,
    WhiteWins,
    BlackWins,
}

impl GameResult {
    /// True if the game result is a win for the given player.
    pub fn is_win_for_player(self, player_color: PlayerColor) -> bool {
        match self {
            GameResult::BlackWins => player_color == PlayerColor::Black,
            GameResult::WhiteWins => player_color == PlayerColor::White,
            _ => false,
        }
    }
}

/// Describes a move a player can make in a game.
/// I.e., in Reversi, a move could be at position (3,7).
pub trait GameAction: Copy + Debug + PartialEq + Display {
    /// Returns true if this GameMove represents a forced turn pass.
    fn is_forced_pass(self) -> bool;
}

/// A trait describing a complete state of some Game,
/// such as the board position, the current player's turn,
/// and other relevant info.
pub trait GameState: Clone + Display {
    /// The type that will be uesd to describe
    /// the actions that players will select during the game.
    type Action: GameAction;

    /// Returns a human-friendly string for representing the state.
    fn human_friendly(&self) -> String;

    /// Sets the current game state to its initial,
    /// ready-to-play position. E.g., a game of Reversi begins
    /// with four pieces already on the board.
    /// Setting those four pieces would be done here.
    fn initialize_board(&mut self);

    /// Returns a fresh, ready-to-play game state for this game.
    /// Implementors probably want to use initialize_board() to achieve this.
    fn initial_state() -> Self;

    /// Returns the possible moves the given player can make for the current state.
    fn legal_moves(&self, player: PlayerColor) -> &[Self::Action];

    /// Apply the given move (or 'action') to this state, mutating this state
    /// and advancing it to the resulting state.
    fn apply_move(&mut self, action: Self::Action);

    /// Returns the player color whose turn it currently is.
    fn current_player_turn(&self) -> PlayerColor;

    /// Returns the score of the given player in this state.
    /// Note that in some games, this may be meaningless,
    /// as there is no running score over the course of the game,
    /// but only a winner and loser determined at the very end.
    fn player_score(&self, player: PlayerColor) -> usize;

    /// Given a legal move (or 'action'), returns the resulting state of applying the action
    /// to this state, without mutating the original state.
    /// This is done by cloning and then invoking apply_move().
    fn next_state(&self, action: Self::Action) -> Self {
        let mut cloned = self.clone();
        cloned.apply_move(action);

        cloned
    }

    /// Skip the current player's turn without taking any action.
    /// Advances to the next player's turn.
    fn skip_turn(&mut self);

    /// True if the game is over; i.e., the win condition has been met, or neither player can take any further action.
    fn is_game_over(&self) -> bool;

    /// The GameResult, or None if the game is not yet over.
    /// The default implementation selects the winner with the highest value
    /// for player_score().
    fn game_result(&self) -> Option<GameResult> {
        if !self.is_game_over() {
            return None;
        }

        let white_score = self.player_score(PlayerColor::White);
        let black_score = self.player_score(PlayerColor::Black);

        if white_score > black_score {
            Some(GameResult::WhiteWins)
        } else if black_score > white_score {
            Some(GameResult::BlackWins)
        } else {
            Some(GameResult::Tie)
        }
    }

    /// Apply the given moves (or 'actions') to this state, mutating it
    /// each time and advancing it through the chain of states.
    /// Implemented in terms of apply_move().
    fn apply_moves(&mut self, moves: impl IntoIterator<Item = Self::Action>) {
        for m in moves {
            self.apply_move(m);
        }
    }
}

/// A trait representing the functionality of a GameAgent.
/// Most importantly, given a GameState, a GameAgent must be able to decide a GameMove.
pub trait GameAgent<TState: GameState> {
    /// Given the state and slice of legal moves,
    /// the agent will respond with its selected move for
    /// the player to take.
    fn pick_move(&self, state: &TState, legal_moves: &[TState::Action]) -> TState::Action;

    /// Invoked by the game runner to allow both agents
    /// to observe actions taken over the course of the game.
    /// A default implementation is provided that does nothing,
    /// so implementors can ignore this if they have no need for it.
    fn observe_action(&self, _player: PlayerColor, _action: TState::Action, _result: &TState) {}

    /// Returns the player color of this player.
    fn player_color(&self) -> PlayerColor;
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn is_win_for_player_expects_valid_result() {
        let tie = GameResult::Tie;
        assert!(!tie.is_win_for_player(PlayerColor::White));
        assert!(!tie.is_win_for_player(PlayerColor::Black));

        let black_wins = GameResult::BlackWins;
        assert!(!black_wins.is_win_for_player(PlayerColor::White));
        assert!(black_wins.is_win_for_player(PlayerColor::Black));

        let white_wins = GameResult::WhiteWins;
        assert!(white_wins.is_win_for_player(PlayerColor::White));
        assert!(!white_wins.is_win_for_player(PlayerColor::Black));
    }
}
