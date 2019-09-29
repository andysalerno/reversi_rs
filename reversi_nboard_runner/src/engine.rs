use crate::util::{log, Log, NboardError};
use lib_agents::{MctsAgent, RandomAgent};
use lib_boardgame::{GameAgent, GameState, PlayerColor};
use lib_reversi::reversi::Reversi;
use lib_reversi::reversi_gamestate::ReversiState;
use lib_reversi::{BoardPosition, ReversiPlayerAction};
use std::error::Error;
use std::io::{self, Read, Write};

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
            ReversiPlayerAction::PassTurn => return NBoardAction(String::new()),
            ReversiPlayerAction::Move { position } => (position.col(), position.row()),
        };

        let letter_col = match x_pos {
            0 => 'a',
            1 => 'b',
            2 => 'c',
            3 => 'd',
            4 => 'e',
            5 => 'f',
            6 => 'g',
            7 => 'h',
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

    log(Log::Info(format!("Exiting.")));
}

pub fn run_loop() -> Result<(), Box<dyn Error>> {
    let mut black = MctsAgent::<ReversiState>::new(PlayerColor::Black);
    let mut white = MctsAgent::<ReversiState>::new(PlayerColor::White);
    // let mut black = RandomAgent;
    // let white = RandomAgent;

    let mut state = ReversiState::initial_state();

    let mut move_count = 0;

    loop {
        let msg = read_from_stdin()?;
        log(Log::Info(format!("Received raw msg: {}", msg.trim())));

        let parsed = parse_msg(&msg)?;
        log(Log::Info(format!("Parsed message as: {:?}", parsed)));

        match parsed {
            MsgFromGui::Ping(n) => writeln_to_stdout(format!("pong {}", n))?,
            MsgFromGui::Move(m) => {
                let reversi_move = nboard_action_to_reversi_action(NBoardAction(m));
                apply_action_and_observe(&mut state, reversi_move, &mut black, &mut white);
                move_count += 1;
            }
            MsgFromGui::SetGame(ggf) => {
                let mut history = parse_game_history(&ggf);
                history.drain(..move_count);

                for m in &history {
                    log(Log::Info(format!("Saw move: {}", m)));
                    apply_action_and_observe(&mut state, *m, &mut black, &mut white);
                    log(Log::Info(format!(
                        "Next state:\n{}",
                        state.human_friendly()
                    )));
                }

                move_count += history.len();
            }
            MsgFromGui::Go => {
                log(Log::Info("Running agent to select move...".to_owned()));

                let cur_player = state.current_player_turn();
                let selected_move = match cur_player {
                    PlayerColor::Black => {
                        black.pick_move(&state, state.legal_moves(PlayerColor::Black))
                    }
                    PlayerColor::White => {
                        white.pick_move(&state, state.legal_moves(PlayerColor::White))
                    }
                };

                let nboard_action: NBoardAction = selected_move.into();

                let agent_name = match cur_player {
                    PlayerColor::Black => "Black",
                    PlayerColor::White => "White",
                };

                log(Log::Info(format!(
                    "Agent {} picked {} (in NBoard lingo: {})",
                    agent_name, selected_move, nboard_action.0
                )));

                writeln_to_stdout(format!("=== {}", nboard_action.0))?;
            }
            _ => {}
        }
    }
}

fn apply_action_and_observe(
    state: &mut ReversiState,
    action: ReversiPlayerAction,
    black: &mut impl GameAgent<ReversiState>,
    white: &mut impl GameAgent<ReversiState>,
) {
    let player_turn = state.current_player_turn();
    state.apply_move(action);
    black.observe_action(player_turn, action, &state);
    white.observe_action(player_turn, action, &state);
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
        ["move", m] => MsgFromGui::Move(m.to_string()),
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

fn parse_game_history(ggf: &str) -> Vec<ReversiPlayerAction> {
    // (;GM[Othello]PC[NBoard]DT[2019-09-29 03:22:14 GMT]PB[Andy]PW[rustrs]RE[?]TI[5:00]TY[8]BO[8 ---------------------------O*------*O--------------------------- *]B[C4//5.558]W[C3]B[F5//26.906];)

    let mut result = Vec::new();
    let mut s = String::from(ggf);

    loop {
        let next_move_idx = {
            let next_b_move = s.find("]B[");
            let next_w_move = s.find("]W[");

            if next_b_move.is_some() && next_w_move.is_some() {
                Some(usize::min(next_b_move.unwrap(), next_w_move.unwrap()))
            } else {
                next_b_move.or(next_w_move)
            }
        };

        match next_move_idx {
            Some(idx) => {
                s.drain(..idx);

                // ']B[' or ']W['
                let color_str: String = s.drain(..3).collect();
                let player_color = match color_str.chars().nth(1).expect("must match ]B[ pattern") {
                    'B' => PlayerColor::Black,
                    'W' => PlayerColor::White,
                    c => panic!("Expected 'B' or 'W', saw: {}", c),
                };

                // C4, F5, etc
                let ggf_move: String = s.drain(..2).collect();
                let ggf_move = NBoardAction(ggf_move);
                let reversi_action = nboard_action_to_reversi_action(ggf_move);
                result.push(reversi_action);
            }
            None => return result,
        }
    }

    result
}

fn nboard_action_to_reversi_action(n: NBoardAction) -> ReversiPlayerAction {
    let letter = n.0.chars().nth(0).expect("move_str first letter");
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

    let y_pos_val =
        n.0.chars()
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

fn writeln_to_stdout<T: AsRef<str>>(s: T) -> Result<(), Box<dyn Error>> {
    log(Log::Info(format!("Sending message: {}", s.as_ref())));
    let with_newline = format!("{}\n", s.as_ref());
    io::stdout().write_all(with_newline.as_bytes())?;
    io::stdout().flush()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_game_history_finds_one_move() {
        let ggf_string = r"(;GM[Othello]PC[NBoard]DT[2019-09-25 06:42:54 GMT]PB[Andy]PW[]RE[?]TI[5:00]TY[8]BO[8 ---------------------------O*------*O--------------------------- *]B[D3//2.991];)";

        let parsed_move = parse_game_history(ggf_string)
            .iter()
            .last()
            .unwrap()
            .clone();

        match parsed_move {
            ReversiPlayerAction::Move { position } => {
                assert_eq!(position, BoardPosition::new(3, 5))
            }
            _ => panic!("Expected to find a board position."),
        }
    }

    #[test]
    fn parse_game_history_finds_all_moves() {
        let ggf_string = r"(;GM[Othello]PC[NBoard]DT[2019-09-29 03:22:14 GMT]PB[Andy]PW[rustrs]RE[?]TI[5:00]TY[8]BO[8 ---------------------------O*------*O--------------------------- *]B[C4//5.558]W[C3]B[F5//26.906];)";

        let history = parse_game_history(ggf_string);

        match history[0] {
            ReversiPlayerAction::Move { position } => {
                assert_eq!(position, BoardPosition::new(2, 4))
            }
            _ => panic!("Expected to find a board position."),
        }

        match history[1] {
            ReversiPlayerAction::Move { position } => {
                assert_eq!(position, BoardPosition::new(2, 5))
            }
            _ => panic!("Expected to find a board position."),
        }

        match history[2] {
            ReversiPlayerAction::Move { position } => {
                assert_eq!(position, BoardPosition::new(5, 3))
            }
            _ => panic!("Expected to find a board position."),
        }

        assert_eq!(3, history.len());
    }

    #[test]
    fn nboard_action_from_reversi_action() {
        let bottom_left_position = ReversiPlayerAction::Move {
            position: BoardPosition::new(0, 0),
        };

        let nboard_bot_left: NBoardAction = bottom_left_position.into();
        assert_eq!(nboard_bot_left.0, "a8".to_owned());

        let top_right_position = ReversiPlayerAction::Move {
            position: BoardPosition::new(7, 7),
        };

        let nboard_top_right: NBoardAction = top_right_position.into();
        assert_eq!(nboard_top_right.0, "h1".to_owned());

        let one_one = ReversiPlayerAction::Move {
            position: BoardPosition::new(1, 1),
        };

        let nboard_one_one: NBoardAction = one_one.into();
        assert_eq!(nboard_one_one.0, "b7".to_owned());
    }
}
