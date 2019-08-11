#[allow(unused)]
use lib_agents::human_agent::HumanAgent;

#[allow(unused)]
use lib_tic_tac_toe::tic_tac_toe::TicTacToe;

use lib_agents::mcts_agent::MctsAgent;
use lib_boardgame::{Game, PlayerColor, GameResult};
use lib_reversi::reversi::Reversi;
use lib_reversi::reversi_gamestate::ReversiState;
use lib_tic_tac_toe::tic_tac_toe_gamestate::TicTacToeState;

fn main() {
    let game_count: usize = std::env::args()
        .nth(1).unwrap_or_else(|| "1".into())
        .parse().expect("Couldn't parse arg as a usize.");

    let results = (0..game_count)
        .map(|_| play_reversi())
        .collect::<Vec<_>>();

    let white_wins = results.iter().filter(|&&r| r == GameResult::WhiteWins).count();
    let black_wins = results.iter().filter(|&&r| r == GameResult::BlackWins).count();
    let ties = results.iter().filter(|&&r| r == GameResult::Tie).count();

    let total = results.len();

    println!("Black wins: {} ({:.2})", black_wins, black_wins as f32 / total as f32);
    println!("White wins: {} ({:.2})", white_wins, white_wins as f32 / total as f32);
    println!("Ties      : {} ({:.2})", ties, ties as f32 / total as f32);
}

#[allow(unused)]
fn play_reversi() -> lib_boardgame::GameResult {
    // let white = HumanAgent::new(PlayerColor::White);
    let black = MctsAgent::<ReversiState>::new(PlayerColor::Black);
    let white = MctsAgent::<ReversiState>::new(PlayerColor::White);

    let mut game = Reversi::new(white, black);

    game.play_to_end()
}

#[allow(unused)]
fn play_tic_tac_toe() -> lib_boardgame::GameResult {
    let black = MctsAgent::<TicTacToeState>::new(PlayerColor::Black);
    // let black = HumanAgent::new(PlayerColor::Black);

    let white = MctsAgent::<TicTacToeState>::new(PlayerColor::White);
    // let white = HumanAgent::new(PlayerColor::White);

    let mut game = TicTacToe::new(white, black);

    game.play_to_end()
}