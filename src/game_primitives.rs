pub enum PlayerColor {
    Black,
    White,
}

pub enum GameResult {
    NotOver,
    Tie,
    WhiteWins,
    BlackWins,
}

pub trait PrintableGameState {}

pub trait PlayableGame {
    /// Returns the player whose turn it is.
    fn whose_turn(&self) -> PlayerColor;

    fn is_game_over(&self) -> bool;

    fn game_result(&self) -> GameResult;

    fn player_take_turn(&mut self);

    /// Returns a human-friendly string representation of the current game state.
    fn printable_state(&self) -> std::convert::Into<&str>;
}

struct Test;

impl PlayableGame for Test {
    fn whose_turn(&self) -> PlayerColor {
        PlayerColor::White
    }
}