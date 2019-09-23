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

pub fn run() -> Result<(), Box<dyn Error>> {
    loop {
        let msg = read_from_stdin()?;
        let parsed = parse_msg(&msg);

        println!("Received message from gui: {:?}", parsed);
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
        ["nboard", "version:"] => MsgFromGui::Go,
        ["set", "depth"] => MsgFromGui::SetDepth(0),
        ["set", "game"] => MsgFromGui::SetGame("123".into()),
        ["set", "contempt"] => MsgFromGui::SetContempt(0),
        ["move"] => MsgFromGui::Move("123".into()),
        ["hint"] => MsgFromGui::Hint(0),
        ["go"] => MsgFromGui::Go,
        ["ping"] => MsgFromGui::Ping(0),
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
    std::io::stdin().read_to_string(&mut buffer)?;

    Ok(buffer)
}
