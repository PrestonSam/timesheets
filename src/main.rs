use std::{fs::read_to_string, path::{Path, PathBuf}};

use clap::Parser;
use cli::{parse_cli, Action, TshArgs};
use evaluator::evaluate_timesheets;
use parser::{parse_timesheets, ParsingError};

mod cli;
mod parser;
mod evaluator;
mod utils;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    file_path: PathBuf,
}

// TODO new features:
/* Open given log-type with given time (or current, if not specified)
 * End open log-type with given time (or current, if not specified)
 * Have a special flag to end the working day with the previous command
 */

#[derive(Debug)]
enum TimesheetsError {
    FileReadError(std::io::Error),
    ParsingError(ParsingError),
}

impl std::fmt::Display for TimesheetsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TimesheetsError::FileReadError(err) => err.fmt(f),
            TimesheetsError::ParsingError(err) => err.fmt(f),
        }
    }
}

fn run_timesheets(path: &Path) -> Result<(), TimesheetsError> {
    let code = read_to_string(path)
        .map_err(TimesheetsError::FileReadError)?;

    let timesheets = parse_timesheets(&code)
        .map_err(TimesheetsError::ParsingError)?;

    let total_delta = evaluate_timesheets(timesheets);

    print!("{total_delta}");

    Ok(())
}

fn main() {
    match parse_cli() {
        TshArgs { file_path, command: None } => 
            match run_timesheets(&file_path) {
                Ok(_) => (),
                Err(err) => eprintln!("{}", err),
            }

        // Still experimental
        TshArgs { file_path: _file_path, command: Some(command) } => {
            match command {
                Action::Start { log_type, time_range } =>
                    println!("{log_type:?}: {time_range}"),

                Action::End { log_type, time_range } =>
                    println!("{log_type:?}: {time_range}"),
            }
        }
    }
}
