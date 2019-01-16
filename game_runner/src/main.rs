use lib_boardgame::game_primitives::{Game, PlayerColor};
use lib_reversi::agents::random_agent::RandomAgent;
use lib_reversi::reversi::Reversi;

fn main() {
    let white = RandomAgent::new(PlayerColor::White);
    let black = RandomAgent::new(PlayerColor::Black);

    let mut game = Reversi::new(white, black);

    game.play_to_end();
}
