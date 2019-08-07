use std::fmt;
use std::str::FromStr;
pub mod test_impls;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum PlayerColor {
    Black,
    White,
}

impl PlayerColor {
    pub fn opponent(self) -> Self {
        match self {
            PlayerColor::Black => PlayerColor::White,
            PlayerColor::White => PlayerColor::Black,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum GameResult {
    Tie,
    WhiteWins,
    BlackWins,
}

impl GameResult {
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
pub trait GameMove: Copy + fmt::Debug + Send + PartialEq {}

pub trait GameMoveFromStr: GameMove + FromStr {
    fn from_str(s: &str, player_color: PlayerColor) -> Result<Self, Self::Err>;
}

/// Describes a complete state of some Game,
/// such as the board position, the current player's turn,
/// or any other relevant info.
pub trait GameState: Clone + Send {
    type Move: GameMove;

    /// Returns a human-friendly string for representing the state.
    fn human_friendly(&self) -> String;

    /// Gives the implementation a chance to initialize the starting state of a game
    /// before gameplay begins.
    fn initialize_board(&mut self);

    /// Returns a fresh, ready-to-play game state for this game.
    fn initial_state() -> Self;

    /// Returns the possible moves the given player can make for the current state.
    fn legal_moves(&self, player: PlayerColor) -> Vec<Self::Move>;

    /// Apply the given move (or 'action') to this state, mutating this state
    /// and advancing it to the resulting state.
    fn apply_move(&mut self, action: Self::Move);

    /// Returns the current player whose turn it currently is.
    fn current_player_turn(&self) -> PlayerColor;

    /// Returns the score of the given player in this state.
    fn player_score(&self, player: PlayerColor) -> usize;

    /// Given a legal move (or 'action'), return the resulting state of applying the action
    /// to this state (does not mutate this state).
    fn next_state(&self, action: Self::Move) -> Self {
        let mut cloned = self.clone();
        cloned.apply_move(action);

        cloned
    }

    /// Skip the current player's turn without taking any action.
    /// Advances to the next player's turn.
    fn skip_turn(&mut self);

    /// True if the game is over (i.e. the win condition has been met, or neither player can take any further action).
    fn is_game_over(&self) -> bool;

    /// The GameResult, or None if the game is not yet over.
    fn game_result(&self) -> Option<GameResult> {
        let white_score = self.player_score(PlayerColor::White);
        let black_score = self.player_score(PlayerColor::Black);

        if !self.is_game_over() {
            None
        } else if white_score > black_score {
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
    fn apply_moves(&mut self, moves: impl IntoIterator<Item = Self::Move>) {
        for m in moves {
            self.apply_move(m);
        }
    }
}

pub trait Game<WhiteAgent, BlackAgent>
where
    WhiteAgent: GameAgent<Self::State>,
    BlackAgent: GameAgent<Self::State>,
{
    type State: GameState;

    fn white_agent(&self) -> &WhiteAgent;
    fn black_agent(&self) -> &BlackAgent;

    /// The game's current state.
    fn game_state(&self) -> &Self::State;

    /// The game's current state.
    fn game_state_mut(&mut self) -> &mut Self::State;

    /// True if the the game has ended, either due to a forced win,
    /// draw, or forfeit.
    fn is_game_over(&self) -> bool;

    /// The GameResult, or None if the game is not yet over.
    fn game_result(&self) -> Option<GameResult>;

    /// Invokes the current player agent to pick a move,
    /// then updates the game state with the result.
    fn player_take_turn(&mut self, player: PlayerColor) {
        let state = self.game_state();
        let legal_moves = state.legal_moves(player);

        let picked_action = match player {
            PlayerColor::Black => self.black_agent().pick_move(state, &legal_moves),
            PlayerColor::White => self.white_agent().pick_move(state, &legal_moves),
        };

        if legal_moves.iter().find(|&&m| m == picked_action).is_none() {
            panic!("Agent provided a move that is illegal.");
        }

        println!("Player {:?} picked move {:?}", player, picked_action);

        let state = self.game_state_mut();

        state.apply_move(picked_action);
    }

    /// Applies each player's turn one at a time until the game is over,
    /// and returns the game result.
    fn play_to_end(&mut self) -> GameResult {
        self.game_state_mut().initialize_board();
        println!("Board initialized.");

        while !self.is_game_over() {
            println!("{}", self.game_state().human_friendly());
            let cur_player_color = self.game_state().current_player_turn();
            self.player_take_turn(cur_player_color);
        }

        println!("{}", self.game_state().human_friendly());

        self.game_result()
            .expect("The game is over, so there must be a game result.")
    }
}

/// A trait representing the functionality of a GameAgent.
/// Specifically, given a GameState, a GameAgent must be able to decide a GameMove.
pub trait GameAgent<TState: GameState> {
    fn pick_move(&self, state: &TState, legal_moves: &[TState::Move]) -> TState::Move;
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
