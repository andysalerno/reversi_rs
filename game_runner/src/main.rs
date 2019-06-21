use lib_agents::mcts_agent::MCTSRcAgent;
use lib_agents::random_agent::RandomAgent;
use lib_boardgame::{Game, PlayerColor};
use lib_reversi::reversi::Reversi;
use lib_reversi::reversi_gamestate::ReversiState;
use lib_tic_tac_toe::tic_tac_toe::TicTacToe;
use lib_tic_tac_toe::tic_tac_toe_gamestate::TicTacToeState;

fn main() {
    // play_tic_tac_toe();
    play_reversi();
}

#[allow(unused)]
fn play_reversi() {
    let black = MCTSRcAgent::<ReversiState>::new(PlayerColor::Black);
    let white = MCTSRcAgent::<ReversiState>::new(PlayerColor::White);

    let mut game = Reversi::new(white, black);

    let game_result = game.play_to_end();
    println!("Result: {:?}", game_result);
}

#[allow(unused)]
fn play_tic_tac_toe() {
    let black = MCTSRcAgent::<TicTacToeState>::new(PlayerColor::Black);
    let white = MCTSRcAgent::<TicTacToeState>::new(PlayerColor::White);

    let mut game = TicTacToe::new(white, black);

    let game_result = game.play_to_end();

    println!("Result: {:?}", game_result);
}