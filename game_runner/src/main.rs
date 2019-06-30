use lib_agents::human_agent::HumanAgent;
use lib_agents::mcts_agent::MctsAgent;
use lib_boardgame::{Game, PlayerColor};
use lib_reversi::reversi::Reversi;
use lib_reversi::reversi_gamestate::ReversiState;
use lib_tic_tac_toe::tic_tac_toe::TicTacToe;
use lib_tic_tac_toe::tic_tac_toe_gamestate::TicTacToeState;

fn main() {

    // for _ in 0..1000 {
    //     play_tic_tac_toe();
    // }

    // rayon::join(
    //     || (0..500).for_each(|_| play_tic_tac_toe()),
    //     || (0..500).for_each(|_| play_tic_tac_toe()),
    // );

    play_reversi();
}

#[allow(unused)]
fn play_reversi() {
    // let white = HumanAgent::new(PlayerColor::White);
    let white = MctsAgent::<ReversiState>::new(PlayerColor::White);
    let black = MctsAgent::<ReversiState>::new(PlayerColor::Black);

    let mut game = Reversi::new(white, black);

    let game_result = game.play_to_end();
    println!("Result: {:?}", game_result);
}

#[allow(unused)]
fn play_tic_tac_toe() {
    let black = MctsAgent::<TicTacToeState>::new(PlayerColor::Black);
    let white = MctsAgent::<TicTacToeState>::new(PlayerColor::White);

    // let mut game = TicTacToe::new(white, black);

    // let game_result = game.play_to_end();

    // println!("Result: {:?}", game_result);
}