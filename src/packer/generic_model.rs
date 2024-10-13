use std::{fmt::Display, num::ParseIntError};

use itertools::Itertools;
use pest::RuleType;

use super::generic_utils::PackingError;

#[derive(Debug)]
pub struct RuleAndProvidence<R: RuleType>(pub R, pub String);

impl<R: RuleType> Display for RuleAndProvidence<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}({})", self.0, self.1))
    }
}

#[derive(Debug)]
pub enum PackingErrorVariant<R: RuleType> {
    // TODO deal with the optional vars because they're not great. You should have different errors for when there are no expected rules
    WrongRulesSeq { found_rules: Vec<RuleAndProvidence<R>>, expected_rules: Option<Vec<R>> },
    WrongRulesAlt { found_rule: RuleAndProvidence<R>, expected_rules: Vec<R> },
    TooFewRules { present_rules: Vec<RuleAndProvidence<R>>, expected_rules: Option<Vec<R>>, expected_count: usize },
    TooManyRules { present_rules: Vec<RuleAndProvidence<R>>, expected_rules: Option<Vec<R>>, expected_count: usize },
    NoChildrenFound { expected_rules: Option<Vec<R>> },
    ParseUsizeError(ParseIntError),
    TimeParseError(chrono::ParseError),
}

impl<R: RuleType> PackingErrorVariant<R> {
    fn display_vec<T>(key: &str, maybe_values: Option<&Vec<T>>, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    where T: Display
    {
        match maybe_values {
            Some(values) =>
                f.write_fmt(format_args!("  {key}:\n    {}\n", values.iter().join(", "))),

            None =>
                f.write_fmt(format_args!("  {key}:\n    [No rules]\n")),
        }
    }

    fn debug_vec<T>(key: &str, maybe_values: Option<&Vec<T>>, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    where T: std::fmt::Debug
    {
        match maybe_values {
            Some(values) =>
                f.write_fmt(format_args!("  {key}:\n    {}\n", values.iter().map(|v| format!("{v:?}")).join(", "))),

            None =>
                f.write_fmt(format_args!("  {key}:\n    [No rules]\n")),
        }
    }
}

impl<R: RuleType> Display for PackingErrorVariant<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::WrongRulesSeq { found_rules, expected_rules } => {
                f.write_str("Encountered unexpected sequence of rules:\n")?;
                Self::debug_vec("Expected a sequence of the following rules", expected_rules.as_ref(), f)?;
                Self::display_vec("Instead found the following sequence of rules", Some(found_rules), f)?;

                Ok(())
            }

            Self::WrongRulesAlt { found_rule, expected_rules } => {
                f.write_str("Encountered unexpected rule:\n")?;
                Self::debug_vec("Expected any one of the following rules", Some(expected_rules), f)?;
                f.write_fmt(format_args!("  Instead found the following rule {found_rule}"))?;

                Ok(())
            }

            Self::TooFewRules { present_rules, expected_rules, expected_count } => {
                f.write_fmt(format_args!("Found fewer than {} rules:\n", expected_count))?;
                Self::display_vec("Matching rules", Some(present_rules), f)?;
                Self::debug_vec("Expected rules", expected_rules.as_ref(), f)?;

                Ok(())
            }

            Self::TooManyRules { present_rules, expected_rules, expected_count } => {
                f.write_fmt(format_args!("Found more than {} rules:\n", expected_count))?;
                Self::display_vec("Matching rules", Some(present_rules), f)?;
                Self::debug_vec("Expected rules", expected_rules.as_ref(), f)?;

                Ok(())
            }

            Self::NoChildrenFound { expected_rules } => {
                f.write_str("Found no children for node:\n")?;
                Self::debug_vec("Expected rules", expected_rules.as_ref(), f)?;

                Ok(())
            }

            Self::ParseUsizeError(parse_int_err) =>
                f.write_fmt(format_args!("{}", parse_int_err)),

            Self::TimeParseError(chrono_parse_err) =>
                f.write_fmt(format_args!("{}", chrono_parse_err)),
        }
    }
}


pub trait PackingResult<R: RuleType> {
    fn with_rule(self, rule: R) -> Self;
}

impl<T, R: RuleType> PackingResult<R> for Result<T, PackingError<PackingErrorVariant<R>, R>> {
    fn with_rule(self, rule: R) -> Self
    {
        self.map_err(|err| err.with_rule(rule))
    }
}
