use lib_agents::{HumanAgent, MctsAgent, RandomAgent};
use lib_boardgame::{Game, GameResult, GameState, PlayerColor};
use lib_connect_four::{ConnectFour, ConnectFourState};
use lib_printer::{out, out_impl};
use lib_reversi::reversi::Reversi;
use lib_reversi::reversi_gamestate::ReversiState;
use lib_tic_tac_toe::tic_tac_toe::TicTacToe;
use lib_tic_tac_toe::tic_tac_toe_gamestate::TicTacToeState;

#[derive(Debug)]
struct Args {
    game_count: usize,
    start_nboard: bool,
}

fn main() {
    let args = get_args();

    let results = (0..args.game_count)
        .map(|_| play_reversi())
        .collect::<Vec<_>>();

    let white_wins = results
        .iter()
        .filter(|&&r| r == GameResult::WhiteWins)
        .count();
    let black_wins = results
        .iter()
        .filter(|&&r| r == GameResult::BlackWins)
        .count();
    let ties = results.iter().filter(|&&r| r == GameResult::Tie).count();

    let total = results.len();

    out!(
        "Black wins: {} ({:.2})",
        black_wins,
        black_wins as f32 / total as f32
    );
    out!(
        "White wins: {} ({:.2})",
        white_wins,
        white_wins as f32 / total as f32
    );
    out!("Ties      : {} ({:.2})", ties, ties as f32 / total as f32);
}

fn get_args() -> Args {
    // let args = std::env::args().collect::<Vec<_>>();
    let mut args = std::env::args();

    let game_count = args
        .nth(1)
        .unwrap_or_else(|| "1".into())
        .parse::<usize>()
        .unwrap_or_else(|_| 1);

    let start_nboard = args.any(|a| a.to_lowercase() == "nboard");

    Args {
        game_count,
        start_nboard,
    }
}

#[allow(unused)]
fn play_reversi() -> lib_boardgame::GameResult {
    let black = Box::new(MctsAgent::<ReversiState>::new(PlayerColor::Black));
    // let white = Box::new(MctsAgent::<ReversiState>::new(PlayerColor::White));
    // let white = Box::new(HumanAgent::new(PlayerColor::White));
    let white = Box::new(RandomAgent);

    let mut game = Reversi::new(white, black);

    game.play_to_end()
}

#[allow(unused)]
fn play_tic_tac_toe() -> lib_boardgame::GameResult {
    // let black = Box::new(MctsAgent::<TicTacToeState>::new(PlayerColor::Black));
    let black = Box::new(HumanAgent::new(PlayerColor::Black));
    let white = Box::new(MctsAgent::<TicTacToeState>::new(PlayerColor::White));
    // let white = Box::new(HumanAgent::new(PlayerColor::White));

    let mut game = TicTacToe::new(white, black);

    game.play_to_end()
}

#[allow(unused)]
fn play_connect_four() -> lib_boardgame::GameResult {
    let black = Box::new(MctsAgent::<ConnectFourState>::new(PlayerColor::Black));
    let white = Box::new(MctsAgent::<ConnectFourState>::new(PlayerColor::White));

    // let black = Box::new(RandomAgent);
    // let white = Box::new(HumanAgent::new(PlayerColor::White));

    let mut game = ConnectFour::new(white, black);

    out!("{}", game.game_state().human_friendly());

    game.play_to_end()
}
