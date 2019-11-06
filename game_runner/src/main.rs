use lib_agents::{HumanAgent, MctsAgent, RandomAgent};
use lib_boardgame::{GameRunner, GeneralGameRunner, PlayerColor};
use lib_connect_four::ConnectFourState;
use lib_reversi::ReversiState;
use lib_tic_tac_toe::tic_tac_toe_gamestate::TicTacToeState;

fn main() {
    play_reversi();
}

#[allow(unused)]
fn play_reversi() -> lib_boardgame::GameResult {
    let black = MctsAgent::<ReversiState>::new(PlayerColor::Black);
    let white = MctsAgent::<ReversiState>::new(PlayerColor::White);

    GeneralGameRunner::play_to_end(&black, &white)
}

#[allow(unused)]
fn play_tic_tac_toe() -> lib_boardgame::GameResult {
    let black = HumanAgent::new(PlayerColor::Black);
    let white = MctsAgent::<TicTacToeState>::new(PlayerColor::White);

    GeneralGameRunner::play_to_end(&black, &white)
}

#[allow(unused)]
fn play_connect_four() -> lib_boardgame::GameResult {
    let black = MctsAgent::<ConnectFourState>::new(PlayerColor::Black);
    let white = HumanAgent::new(PlayerColor::White);

    GeneralGameRunner::play_to_end(&black, &white)
}
