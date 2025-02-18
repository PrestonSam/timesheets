use pest_derive::Parser;
use pest::{iterators::Pairs, Parser};
use lang_packer_model::generic_utils::DropRules;


#[derive(Parser)]
#[grammar = "src/parser/parser.pest"]
pub struct TimesheetsParser;

pub fn parse(code: &str) -> Result<Pairs<'_, Rule>, ::pest::error::Error<Rule>> {
    TimesheetsParser::parse(Rule::body, code)
}

impl DropRules for Rule {
    fn get_drop_rules(&self) -> Vec<Self> {
        vec![ Rule::TAB ]
    }
}
