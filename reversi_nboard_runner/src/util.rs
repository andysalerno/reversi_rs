use std::error::Error;
use std::fmt::{Debug, Display};
use std::fs::OpenOptions;
use std::io::Write;

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
    let log_file_loc = r"C:\Users\Andy\git_repos\reversi_rs\nboard_log.txt";

    let mut f = OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open(log_file_loc)
        .expect("Couldn't open log file.");

    let bytes_msg = match log {
        Log::Info(l) => format!("[Info] {}\n", l),
        Log::Warning(l) => format!("[Warn] {}\n", l),
        Log::Error(l) => format!("[Error] {}\n", l),
    };

    write!(f, "{}", bytes_msg).expect("Failure writing to log");
    // write!(std::io::stdout(), "{}", bytes_msg);
}
