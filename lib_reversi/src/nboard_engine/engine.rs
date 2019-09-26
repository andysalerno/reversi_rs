use super::util::{log, Log, NboardError};
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
    loop {
        let msg = read_from_stdin()?;
        log(Log::Info(format!("Received raw msg: {}", msg.trim())));

        let parsed = parse_msg(&msg)?;
        log(Log::Info(format!("Parsed message as: {:?}", parsed)));
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

fn get_move_from_GGF(ggf: &str) -> String {
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

    second_chunk_split[0].to_string()
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
    fn get_move_from_GGF_finds_move() {
        let ggf_string = r"(;GM[Othello]PC[NBoard]DT[2019-09-25 06:42:54 GMT]PB[Andy]PW[]RE[?]TI[5:00]TY[8]BO[8 ---------------------------O*------*O--------------------------- *]B[D3//2.991];)";

        let parsed_move = get_move_from_GGF(ggf_string);

        assert_eq!("D3", parsed_move, "Expected to parse out the move value from the GGF string.");
    }
}
