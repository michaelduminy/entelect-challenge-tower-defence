extern crate zombot;
extern crate time;
use time::{PreciseTime, Duration};

use zombot::*;
use zombot::engine::command::Command;

use std::error::Error;

const STATE_PATH: &str = "init_state.json";

const COMMAND_PATH: &str = "command.txt";

use std::fs::File;
use std::io::prelude::*;
use std::process;

fn choose_move(settings: &engine::settings::GameSettings, state: &engine::GameState, start_time: &PreciseTime) -> Command {
    let max_time = Duration::milliseconds(1950);
    strategy::monte_carlo::choose_move(settings, state, start_time, max_time)
}


fn write_command(filename: &str, command: Command) -> Result<(), Box<Error> > {
    let mut file = File::create(filename)?;
    write!(file, "{}", command)?;
    Ok(())
}


fn main() {
    let start_time = PreciseTime::now();
    
    println!("Reading in state.json file");
    let (settings, state) = match json::read_state_from_file(STATE_PATH) {
        Ok(ok) => ok,
        Err(error) => {
            println!("Error while parsing JSON file: {}", error);
            process::exit(1);
        }
    };
    let command = choose_move(&settings, &state, &start_time);

    match write_command(COMMAND_PATH, command) {
        Ok(()) => {}
        Err(error) => {
            println!("Error while writing command file: {}", error);
            process::exit(1);
        }
    }

    println!("Elapsed time: {}", start_time.to(PreciseTime::now()));
}