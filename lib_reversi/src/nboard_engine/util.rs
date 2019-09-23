use std::error::Error;
use std::fmt::{Debug, Display};

pub(super) enum Log {
    Info(String),
    Warning(String),
    Error(String),
}

#[derive(Debug)]
pub(super) struct NboardError {
    msg: String,
}

impl NboardError {
    pub fn err2(msg: impl AsRef<str>) -> Self {
        Self {
            msg: String::from(msg.as_ref()),
        }
    }

    pub fn err<T>(msg: impl AsRef<str>) -> Result<T, Self> {
        Result::Err(Self {
            msg: String::from(msg.as_ref()),
        })
    }
}

impl Display for NboardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl Error for NboardError {}

pub(super) fn log(log: Log) {
    match log {
        Log::Info(l) => println!("[Info] {}", l),
        Log::Warning(l) => println!("[Warn] {}", l),
        Log::Error(l) => eprintln!("[Error] {}", l),
    }
}
