use std::{fs::read_to_string, ops::Not, path::{Path, PathBuf}};

use chrono::{DateTime, Local, TimeDelta};
use clap::Parser;
use cli::{parse_cli, Action, TshArgs};
use evaluator::{evaluate_timesheets, TotalDelta, WeekDelta};
use parser::{parse_timesheets, ParsingError};

mod cli;
mod parser;
mod evaluator;

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

fn get_lunch_deadline(deadline: &DateTime<Local>, week_deltas: &Vec<WeekDelta>) -> Option<DateTime<Local>> {
    week_deltas.last()
        ?.day_deltas.last()
        ?.had_lunch.not()
        .then(|| *deadline + TimeDelta::hours(1))
}

fn announce_deadline(total_delta: TotalDelta) {
    // TODO record earliest start time for day
    // Use that as the reference point to see if you've worked 7.5 hours yet.
    // Figure out the difference and add that to the time.
    let deadline = Local::now() - total_delta.total_delta;
    let deadline_str = deadline.format("%H:%M");

    println!("    ┌───────┐");
    println!("    │ {deadline_str:<5} │ EARLIEST FINISH TIME");

    if let Some(lunch_clause_earliest) = get_lunch_deadline(&deadline, &total_delta.week_deltas).map(|d| d.format("%H:%M")) {
        println!("    │ {lunch_clause_earliest:<5} │ EARLIEST FINISH TIME + LUNCH")
    }

    println!("    ├───────┤");

    let today_delta = total_delta.week_deltas
        .last()
        .and_then(|w| w.day_deltas.last())
        .map(|day| day.delta)
        .unwrap_or_else(|| TimeDelta::zero());

    let today_deadline = Local::now() - today_delta;
    let today_deadline_str = today_deadline.format("%H:%M");

    println!("    │ {today_deadline_str:<5} │ RETAIN CREDIT");

    if let Some(today_lunch_clause) = get_lunch_deadline(&today_deadline, &total_delta.week_deltas).map(|d| d.format("%H:%M")) {
        println!("    │ {today_lunch_clause:<5} │ RETAIN CREDIT + LUNCH");
    }

    println!("    └───────┘")
}

fn run_timesheets(path: &Path) -> Result<(), TimesheetsError> {
    let code = read_to_string(path)
        .map_err(TimesheetsError::FileReadError)?;

    let timesheets = parse_timesheets(&code)
        .map_err(TimesheetsError::ParsingError)?;

    let total_delta = evaluate_timesheets(timesheets);

    print!("{total_delta}");

    announce_deadline(total_delta);

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
