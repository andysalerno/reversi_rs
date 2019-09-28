use crate::util::{log, Log, NboardError};
use lib_agents::MctsAgent;
use lib_boardgame::{GameState, PlayerColor};
use lib_reversi::reversi::Reversi;
use lib_reversi::reversi_gamestate::ReversiState;
use lib_reversi::{BoardPosition, ReversiPlayerAction};
use std::error::Error;
use std::io::{self, Read};

#[derive(Debug)]
enum MsgFromGui {
    NBoard(usize),
    SetDepth(usize),
    SetGame(String),
    SetContempt(usize),
    Move(String),
    Hint(usize),
    Go,
    Ping(usize),
    Learn,
    Analyze,
}

/// Represents an NBoard action, which is structured like so:
/// "A8" is the bottom-left position, "H1" is the top-right.
struct NBoardAction(String);

impl From<ReversiPlayerAction> for NBoardAction {
    fn from(action: ReversiPlayerAction) -> Self {
        let (x_pos, y_pos) = match action {
            ReversiPlayerAction::PassTurn => panic!("Not sure how to represent pass yet"),
            ReversiPlayerAction::Move { position } => (position.col(), position.row()),
        };

        let letter_col = match x_pos {
            0 => 'A',
            1 => 'B',
            2 => 'C',
            3 => 'D',
            4 => 'E',
            5 => 'F',
            6 => 'G',
            7 => 'H',
            _ => panic!("x_pos {} not supported right now", x_pos),
        };

        let num_row = 8 - y_pos;

        let nboard_formatted = format!("{}{}", letter_col, num_row);

        NBoardAction(nboard_formatted)
    }
}

pub fn run() {
    let result = run_loop();

    if result.is_err() {
        log(Log::Error(format!(
            "Execution failed with result: {:?}",
            result
        )));
    }
}

pub fn run_loop() -> Result<(), Box<dyn Error>> {
    let black = MctsAgent::<ReversiState>::new(PlayerColor::Black);
    let white = MctsAgent::<ReversiState>::new(PlayerColor::White);

    let mut state = ReversiState::initial_state();

    loop {
        let msg = read_from_stdin()?;
        log(Log::Info(format!("Received raw msg: {}", msg.trim())));

        let parsed = parse_msg(&msg)?;
        log(Log::Info(format!("Parsed message as: {:?}", parsed)));

        match parsed {
            MsgFromGui::SetGame(ggf) => {
                let reversi_action = get_move_from_ggf(&ggf);
                log(Log::Info(format!("Saw move: {}", &reversi_action)));
                state.apply_move(reversi_action);
                log(Log::Info(format!(
                    "Next state:\n{}",
                    state.human_friendly()
                )));
            }
            _ => {}
        }
    }
}

fn parse_msg(msg: &str) -> Result<MsgFromGui, NboardError> {
    let parsed = match msg
        .split_whitespace()
        .into_iter()
        .collect::<Vec<_>>()
        .as_slice()
    {
        ["nboard", version] => MsgFromGui::NBoard(version.parse::<usize>().unwrap()),
        ["set", "depth", depth_str] => MsgFromGui::SetDepth(depth_str.parse::<usize>().unwrap()),
        ["set", "game", g1, g2, g3, g4, g5] => {
            MsgFromGui::SetGame(format!("{} {} {} {} {}", g1, g2, g3, g4, g5))
        }
        ["set", "contempt"] => MsgFromGui::SetContempt(0),
        ["move"] => MsgFromGui::Move("123".into()),
        ["hint"] => MsgFromGui::Hint(0),
        ["go"] => MsgFromGui::Go,
        ["ping", ping_str] => MsgFromGui::Ping(ping_str.parse::<usize>().unwrap()),
        ["learn"] => MsgFromGui::Learn,
        ["analyze"] => MsgFromGui::Analyze,
        _ => {
            return NboardError::err("testing");
        }
    };

    Ok(parsed)
}

fn get_move_from_ggf(ggf: &str) -> ReversiPlayerAction {
    // Example of GGF:
    // (;GM[Othello]PC[NBoard]DT[2019-09-25 06:42:54 GMT]PB[Andy]PW[]RE[?]TI[5:00]TY[8]BO[8 ---------------------------O*------*O--------------------------- *]B[D3//2.991];)
    //                                                                                                                                                           ^^ That's the last move.

    let split_on_move = ggf.split("]B[").collect::<Vec<_>>();

    if split_on_move.len() <= 1 {
        // pattern not found
        panic!("Couldn't find pattern ']B[' in GGF text: {}", ggf);
    }

    let second_chunk = split_on_move[1];
    let second_chunk_split = second_chunk.split("//").collect::<Vec<_>>();

    if second_chunk_split.len() <= 1 {
        // pattern not found
        panic!("Couldn't find pattern '//' in GGF text: {}", ggf);
    }

    let move_str = second_chunk_split[0].to_string();
    let letter = move_str.chars().nth(0).expect("move_str first letter");
    let x_pos_val = match letter {
        'A' => 0,
        'B' => 1,
        'C' => 2,
        'D' => 3,
        'E' => 4,
        'F' => 5,
        'G' => 6,
        'H' => 7,
        _ => panic!("Didn't recognize board letter: {}", letter),
    };
    let y_pos_val = move_str
        .chars()
        .nth(1)
        .expect("move_str second char")
        .to_string()
        .parse::<usize>()
        .expect("move str from nboard had no y val");
    let y_pos_val = 7 - (y_pos_val - 1);

    let position = lib_reversi::BoardPosition::new(x_pos_val, y_pos_val);
    let action = lib_reversi::ReversiPlayerAction::Move { position };

    action
}

fn read_from_stdin() -> Result<String, Box<dyn Error>> {
    let mut buffer = String::new();
    std::io::stdin().read_line(&mut buffer)?;

    Ok(buffer)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_move_from_ggf_finds_move() {
        let ggf_string = r"(;GM[Othello]PC[NBoard]DT[2019-09-25 06:42:54 GMT]PB[Andy]PW[]RE[?]TI[5:00]TY[8]BO[8 ---------------------------O*------*O--------------------------- *]B[D3//2.991];)";

        let parsed_move = get_move_from_ggf(ggf_string);

        match parsed_move {
            ReversiPlayerAction::Move { position } => {
                assert_eq!(position, BoardPosition::new(3, 5))
            }
            _ => panic!("Expected to find a board position."),
        }
    }

    #[test]
    fn nboard_action_from_reversi_action() {
        let bottom_left_position = ReversiPlayerAction::Move {
            position: BoardPosition::new(0, 0),
        };

        let nboard_bot_left: NBoardAction = bottom_left_position.into();
        assert_eq!(nboard_bot_left.0, "A8".to_owned());


        let top_right_position = ReversiPlayerAction::Move {
            position: BoardPosition::new(7, 7),
        };

        let nboard_top_right: NBoardAction = top_right_position.into();
        assert_eq!(nboard_top_right.0, "H1".to_owned());

        let one_one = ReversiPlayerAction::Move {
            position: BoardPosition::new(1, 1),
        };

        let nboard_one_one: NBoardAction = one_one.into();
        assert_eq!(nboard_one_one.0, "B7".to_owned());
    }
}
