use lib_boardgame::game_primitives::Game;
use lib_reversi::agents::human::HumanAgent;
use lib_reversi::agents::mcts_agent::{MCTSAgent, MCTSRcAgent};
use lib_reversi::agents::random_agent::RandomAgent;
use lib_reversi::reversi::Reversi;
use lib_reversi::reversi_gamestate::ReversiState;

fn main() {
    let white = HumanAgent;
    let black = MCTSRcAgent::<ReversiState>::new();

    let mut game = Reversi::new(white, black);

    let game_result = game.play_to_end();
    println!("Result: {:?}", game_result);
}
