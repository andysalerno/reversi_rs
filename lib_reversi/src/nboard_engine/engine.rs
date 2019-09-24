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
    let msg = msg.to_lowercase();

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

fn read_from_stdin() -> Result<String, Box<dyn Error>> {
    let mut buffer = String::new();
    std::io::stdin().read_line(&mut buffer)?;

    Ok(buffer)
}
