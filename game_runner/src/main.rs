use lib_boardgame::game_primitives::Game;
use lib_reversi::agents::random_agent::RandomAgent;
use lib_reversi::reversi::Reversi;

fn main() {
    let white = RandomAgent;
    let black = RandomAgent;

    let mut game = Reversi::new(white, black);

    game.play_to_end();
}
