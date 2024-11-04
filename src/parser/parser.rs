use pest_derive::Parser;
use pest::{iterators::Pairs, Parser};
use token_packer::generic_utils::DropRules;


#[derive(Parser)]
#[grammar = "src/parser/parser.pest"]
pub struct TimesheetsParser;

pub fn parse_timesheets(code: &str) -> Result<Pairs<'_, Rule>, ::pest::error::Error<Rule>> {
    TimesheetsParser::parse(Rule::body, code)
}

impl DropRules for Rule {
    type Rule = Rule;

    fn get_drop_rules(&self) -> Vec<Rule> {
        vec![ Rule::TAB ]
    }
}
