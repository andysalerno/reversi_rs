use lib_agents::mcts_agent::MCTSRcAgent;
use lib_boardgame::{Game, PlayerColor};
use lib_reversi::reversi::Reversi;
use lib_reversi::reversi_gamestate::ReversiState;

fn main() {
    let black = MCTSRcAgent::<ReversiState>::new(PlayerColor::Black);
    let white = MCTSRcAgent::<ReversiState>::new(PlayerColor::White);

    let mut game = Reversi::new(white, black);

    let game_result = game.play_to_end();
    println!("Result: {:?}", game_result);
}
