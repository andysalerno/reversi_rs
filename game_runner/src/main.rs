use lib_agents::mcts_agent::MCTSRcAgent;
use lib_agents::random_agent::RandomAgent;
use lib_boardgame::{Game, PlayerColor};
use lib_reversi::reversi::Reversi;
use lib_reversi::reversi_gamestate::ReversiState;
use lib_tic_tac_toe::tic_tac_toe::TicTacToe;

fn main() {
    play_tic_tac_toe();
}

#[allow(unused)]
fn play_reversi() {
    let black = MCTSRcAgent::<ReversiState>::new(PlayerColor::Black);
    let white = MCTSRcAgent::<ReversiState>::new(PlayerColor::White);

    let mut game = Reversi::new(white, black);

    let game_result = game.play_to_end();
    println!("Result: {:?}", game_result);
}

fn play_tic_tac_toe() {
    let black = RandomAgent;
    let white = RandomAgent;

    let mut game = TicTacToe::new(white, black);

    let game_result = game.play_to_end();

    println!("Result: {:?}", game_result);
}