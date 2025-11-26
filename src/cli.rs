use std::{path::PathBuf, str::FromStr};

use clap::{Subcommand, ValueEnum};
use itertools::Itertools;
use lang_packer_model::{generic_utils::{PackingError, SyntaxTree}, pack_trees::{HasRule, TokenPacker}};
use pest::Parser;
use thiserror::Error;

use crate::parser::{Rule, Time, TimeRangeEnd, TimesheetsParser};

#[derive(clap::Parser, Debug)]
#[command(version, about)]
pub struct TshArgs {
    pub file_path: PathBuf,

    #[command(subcommand)]
    pub command: Option<Action>,
}

#[derive(Subcommand, Debug)]
pub enum Action {
    /// Mark start of log event
    Start {
        #[arg(value_enum)] 
        log_type: LogType,

        #[arg(value_parser = parse_time)]
        time_range: Time,
    },

    /// Mark end of log event
    End {
        #[arg(value_enum)]
        log_type: LogType,

        #[arg(value_parser = parse_time_range_end)]
        time_range: TimeRangeEnd,
    },
}

#[derive(Error, Debug)]
pub enum RuleParseError {
    #[error("{0}")]
    Parsing(#[from] ::pest::error::Error<Rule>),

    #[error("{0}")]
    ExactlyOne(String),

    #[error("{0}")]
    Packing(#[from] PackingError<Rule>),
}

// This should be abstracted as this is a necessary manner for parsing.
fn parse_struct<S>(s: &str, rule: Rule) -> Result<S, RuleParseError>
where
    S: TokenPacker + HasRule<Rule = Rule>,
{
    let pairs = TimesheetsParser::parse(rule, s)?;

    let tree = pairs.into_iter()
        .map(SyntaxTree::from)
        .exactly_one()
        .map_err(|err| RuleParseError::ExactlyOne(err.to_string()))?;
    
    S::pack(&tree)
        .map_err(RuleParseError::Packing)
}

impl FromStr for Time {
    type Err = RuleParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_struct(s, Rule::TIME)
    }
}

fn parse_time(s: &str) -> Result<Time, String> {
    s.parse::<Time>()
        .map_err(|e| format!("{e}"))
}

impl FromStr for TimeRangeEnd {
    type Err = RuleParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_struct(s, Rule::time_range_end)
    }
}

fn parse_time_range_end(s: &str) -> Result<TimeRangeEnd, String> {
    s.parse::<TimeRangeEnd>()
        .map_err(|e| format!("{e}"))
}

#[derive(ValueEnum, Debug, Clone)]
pub enum LogType {
    WorkingDay,
    Work,
    Break,
    Leave,
    Lunch,
}

pub fn parse_cli() -> TshArgs {
    <TshArgs as clap::Parser>::parse()
}
