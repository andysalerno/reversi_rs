use lib_agents::MctsAgent;
use lib_boardgame::{Game, PlayerColor};
use lib_reversi::reversi::Reversi;
use lib_reversi::reversi_gamestate::ReversiState;

mod engine;
mod util;

fn main() {
    engine::run_loop();
}
