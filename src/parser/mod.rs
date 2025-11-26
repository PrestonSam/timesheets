use pest::iterators::Pairs;
use lang_packer_model::{
    generic_utils::{PackingError, SyntaxTree},
    pack_trees::{unpack_only_tree, TokenPacker}
};

use parser_impl::parse;

mod parser_impl;
mod packer;

pub use parser_impl::{TimesheetsParser, Rule};
pub use packer::*;

#[derive(Debug)]
pub enum ParsingError {
    PestError(Box<::pest::error::Error<Rule>>),
    PackingError(PackingError<Rule>),
}

impl std::fmt::Display for ParsingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParsingError::PestError(err) => err.fmt(f),
            ParsingError::PackingError(err) => err.fmt(f),
        }
    }
}

pub fn parse_timesheets(code: &str) -> Result<Weeks, ParsingError> {
    let pairs = parse(code)
        .map_err(ParsingError::PestError)?;

    pack(pairs)
        .map_err(ParsingError::PackingError)
}

pub fn pack(pairs: Pairs<'_, Rule>) -> Result<Weeks, PackingError<Rule>> {
    let trees: Vec<_> = pairs.map(SyntaxTree::from)
        .collect();

    unpack_only_tree(&trees)
        .and_then(Body::pack)
        .map(|body| body.0)
}

