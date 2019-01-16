pub mod agents;
pub mod reversi;

#[cfg(test)]
mod tests {
    use crate::agents::random_agent::RandomAgent;
    use lib_boardgame::game_primitives::{Game, PlayerColor};
    use crate::reversi::reversi::Reversi;

    #[test]
    fn create_game() {
        let white = RandomAgent::new(PlayerColor::White);
        let black = RandomAgent::new(PlayerColor::Black);

        let mut game = Reversi::new(white, black);
        game.play_to_end();
    }
}
