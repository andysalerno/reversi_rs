mod engine;
mod util;

use util::{log, Log};

fn main() {
    let result = engine::run_loop();

    if result.is_err() {
        log(Log::Error(format!(
            "Execution failed with result: {:?}",
            result
        )));
    }

    log(Log::Info("Exiting.".to_string()));
}
