use std::{fs::read_to_string, ops::Not, path::PathBuf};

use chrono::{DateTime, Local, TimeDelta};
use clap::Parser;
use evaluator::{evaluate_timesheets, EvalError, TotalDelta, WeekDelta};
use packer::pack;
use parser::{parse_timesheets, Rule};
use token_packer::{generic_model::PackingErrorVariant, generic_utils::PackingError};

mod parser;
mod packer;
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
    ParsingError(::pest::error::Error<Rule>),
    PackingError(PackingError<PackingErrorVariant<Rule>, Rule>),
    EvalError(EvalError),
}

fn get_lunch_deadline(deadline: &DateTime<Local>, week_deltas: &Vec<WeekDelta>) -> Option<DateTime<Local>> {
    week_deltas.last()
        ?.day_deltas.last()
        ?.had_lunch.not()
        .then(|| *deadline + TimeDelta::hours(1))
}

fn announce_deadline(total_delta: TotalDelta) {
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

fn main() -> Result<(), TimesheetsError> {
    let args = Args::parse();

    let code = read_to_string(args.file_path).map_err(TimesheetsError::FileReadError)?;

    let timesheets = parse_timesheets(&code)
        .inspect_err(|err| eprintln!("{err}"))
        .map_err(TimesheetsError::ParsingError)?;

    let weeks = pack(timesheets)
        .inspect_err(|err| eprintln!("{err}"))
        .map_err(TimesheetsError::PackingError)?;

    let total_delta = evaluate_timesheets(weeks)
        .map_err(TimesheetsError::EvalError)?;

    print!("{total_delta}");

    announce_deadline(total_delta);

    Ok(())
}
