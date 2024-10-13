use chrono::NaiveTime;
// use lang_packer::Packer;

use crate::parser::Rule;

use super::{
    generic_model::PackingErrorVariant, generic_utils::SyntaxTree, pack_trees::{
        ensure_no_more_trees, get_only_tree_child, get_tree_children, get_tree_src_string, make_wrong_rules_alt_error,
        pack_next_tree, unpack_tree_pack_1_child, unpack_tree_pack_2_children, unpack_tree_pack_each_child, unpack_tree_pack_maybe_2_children,
        HasRule, TokenPacker, TokenRepacker
    }, PackingError
};

#[derive(Debug)]
pub struct Date(pub String);

impl HasRule for Date {
    type Rule = crate::parser::Rule;

    fn get_rule() -> Rule {
        Rule::DATE
    }
}

impl TokenPacker for Date {
    fn pack(tree: SyntaxTree<'_, Self::Rule>) -> Result<Self, PackingError<PackingErrorVariant<Self::Rule>, Self::Rule>> {
        get_tree_src_string(tree, Self::get_rule())
            .map(Self)
    }
}

#[derive(Debug)]
pub struct Weekday(pub String);

impl HasRule for Weekday {
    type Rule = crate::parser::Rule;

    fn get_rule() -> Rule {
        Rule::WEEKDAY
    }
}

impl TokenPacker for Weekday {
    fn pack(tree: SyntaxTree<'_, Self::Rule>) -> Result<Self, PackingError<PackingErrorVariant<Self::Rule>, Self::Rule>> {
        get_tree_src_string(tree, Self::get_rule())
            .map(Self)
    }
}

#[derive(Debug)]
pub struct Summary;

impl HasRule for Summary {
    type Rule = crate::parser::Rule;

    fn get_rule() -> Rule {
        Rule::SUMMARY
    }
}

impl TokenPacker for Summary {
    fn pack(tree: SyntaxTree<'_, Self::Rule>) -> Result<Self, PackingError<PackingErrorVariant<Self::Rule>, Self::Rule>> {
        get_tree_src_string(tree, Self::get_rule())?;

        Ok(Summary)
    }
}

#[derive(Debug)]
pub struct TimeStr(String);

impl HasRule for TimeStr {
    type Rule = crate::parser::Rule;

    fn get_rule() -> Rule {
        Rule::TIME
    }
}

impl TokenPacker for TimeStr {
    fn pack(tree: SyntaxTree<'_, Self::Rule>) -> Result<Self, PackingError<PackingErrorVariant<Self::Rule>, Self::Rule>> {
        get_tree_src_string(tree, Self::get_rule())
            .map(Self)
    }
}

#[derive(Debug)]
pub struct Time(pub NaiveTime);

impl HasRule for Time {
    type Rule = crate::parser::Rule;

    fn get_rule() -> Rule {
        Rule::TIME
    }
}

impl TokenRepacker for Time {
    type Packer = TimeStr;

    fn repack(time_str: TimeStr) -> Result<Self, PackingError<PackingErrorVariant<Self::Rule>, Self::Rule>> {
        let TimeStr(time_str) = time_str;

        NaiveTime::parse_from_str(&time_str, "%H:%M")
            .map(Self)
            .map_err(PackingErrorVariant::TimeParseError)
            .map_err(PackingError::new)
    }
}

#[derive(Debug)]
pub struct Numbers(pub usize);

impl HasRule for Numbers {
    type Rule = crate::parser::Rule;

    fn get_rule() -> Rule {
        Rule::numbers
    }
}

impl TokenPacker for Numbers {
    fn pack(tree: SyntaxTree<'_, Self::Rule>) -> Result<Self, PackingError<PackingErrorVariant<Self::Rule>, Self::Rule>> {
        tree.as_string().parse().map_err(PackingErrorVariant::ParseUsizeError)
            .map_err(PackingError::new) 
            .map(Self)
    }
}

#[derive(Debug)]
pub struct Minutes(pub Numbers);

impl HasRule for Minutes {
    type Rule = crate::parser::Rule;

    fn get_rule() -> Rule {
        Rule::PERIOD_MINUTES
    }
}

impl TokenPacker for Minutes {
    fn pack(tree: SyntaxTree<'_, Self::Rule>) -> Result<Self, PackingError<PackingErrorVariant<Self::Rule>, Self::Rule>> {
        unpack_tree_pack_1_child(tree, Self::get_rule())
            .map(Self)
    }
}

#[derive(Debug)]
pub struct Hours(pub Numbers);

impl HasRule for Hours {
    type Rule = crate::parser::Rule;

    fn get_rule() -> Rule {
        Rule::PERIOD_HOURS
    }
}

impl TokenPacker for Hours {
    fn pack(tree: SyntaxTree<'_, Self::Rule>) -> Result<Self, PackingError<PackingErrorVariant<Self::Rule>, Self::Rule>> {
        unpack_tree_pack_1_child(tree, Self::get_rule())
            .map(Self)
    }
}

#[derive(Debug)]
pub struct HoursMinutes(pub Hours, pub Minutes);

impl HasRule for HoursMinutes {
    type Rule = crate::parser::Rule;
    
    fn get_rule() -> Rule {
        Rule::period_hours_minutes
    }
}

impl TokenPacker for HoursMinutes {
    fn pack(tree: SyntaxTree<'_, Self::Rule>) -> Result<Self, PackingError<PackingErrorVariant<Self::Rule>, Self::Rule>> {
        let (hours, minutes) = unpack_tree_pack_2_children(tree, Self::get_rule())?;

        Ok(Self(hours, minutes))
    }
}

#[derive(Debug)]
pub enum Period { // TODO use real periods here. Have custom parsing logic to fetch
    Minutes(Minutes),
    HoursMinutes(HoursMinutes),
}

impl HasRule for Period {
    type Rule = crate::parser::Rule;
    
    fn get_rule() -> Rule {
        Rule::PERIOD
    }
}

impl TokenPacker for Period {
    fn pack(tree: SyntaxTree<'_, Self::Rule>) -> Result<Self, PackingError<PackingErrorVariant<Self::Rule>, Self::Rule>> {
        match get_only_tree_child(tree, Self::get_rule())? {
            t if Minutes::is_packable(&t) =>
                Minutes::pack(t).map(Self::Minutes),
            
            t if HoursMinutes::is_packable(&t) =>
                HoursMinutes::pack(t).map(Self::HoursMinutes),
            
            t => {
                let expected_rules = vec![
                    Minutes::get_rule(),
                    HoursMinutes::get_rule()
                ];

                Err(make_wrong_rules_alt_error(t, expected_rules))
            }
        }
    }
}

#[derive(Debug)]
pub enum TimeRangeEnd {
    Now,
    Time(Time),
}

impl HasRule for TimeRangeEnd {
    type Rule = crate::parser::Rule;
    
    fn get_rule() -> Rule {
        Rule::time_range_end
    }
}

impl TokenPacker for TimeRangeEnd {
    fn pack(tree: SyntaxTree<'_, Self::Rule>) -> Result<Self, PackingError<PackingErrorVariant<Self::Rule>, Self::Rule>> {
        match get_only_tree_child(tree, Self::get_rule())? {
            t if TimeStr::is_packable(&t) =>
                Time::pack(t).map(Self::Time),

            // TODO might want to revisit this case. You should ideally move this logic into a dummy NOW token that's omitted from the output
            SyntaxTree { rule: Rule::NOW, .. } =>
                Ok(Self::Now),
            
            t => {
                let expected_rules = vec![
                    TimeStr::get_rule(),
                    Rule::NOW,
                ];

                Err(make_wrong_rules_alt_error(t, expected_rules))
            }
        }
    }
}

#[derive(Debug)]
pub struct TimeRange(pub Time, pub TimeRangeEnd);

impl HasRule for TimeRange {
    type Rule = crate::parser::Rule;
    
    fn get_rule() -> Rule {
        Rule::time_range
    }
}

impl TokenPacker for TimeRange {
    fn pack(tree: SyntaxTree<'_, Self::Rule>) -> Result<Self, PackingError<PackingErrorVariant<Self::Rule>, Self::Rule>> {
        let (time, time_range_end) = unpack_tree_pack_2_children(tree, Self::get_rule())?;

        Ok(TimeRange(time, time_range_end))
    }
}

#[derive(Debug)]
pub enum TimePeriod {
    Period(Period),
    TimeRange(TimeRange),
}

impl HasRule for TimePeriod {
    type Rule = crate::parser::Rule;
    
    fn get_rule() -> Rule {
        Rule::time_period
    }
}

impl TokenPacker for TimePeriod {
    fn pack(tree: SyntaxTree<'_, Self::Rule>) -> Result<Self, PackingError<PackingErrorVariant<Self::Rule>, Self::Rule>> {
        match get_only_tree_child(tree, Self::get_rule())? {
            t if Period::is_packable(&t) =>
                Period::pack(t).map(Self::Period),
            
            t if TimeRange::is_packable(&t) =>
                TimeRange::pack(t).map(Self::TimeRange),
            
            t => {
                let expected_rules = vec![
                    Period::get_rule(),
                    TimeRange::get_rule(),
                ];

                Err(make_wrong_rules_alt_error(t, expected_rules))
            }
        }
    }
}

#[derive(Debug)]
pub struct WorkLog(pub TimePeriod);

impl HasRule for WorkLog {
    type Rule = crate::parser::Rule;
    
    fn get_rule() -> Rule {
        Rule::work
    }
}

impl TokenPacker for WorkLog {
    fn pack(tree: SyntaxTree<'_, Self::Rule>) -> Result<Self, PackingError<PackingErrorVariant<Self::Rule>, Self::Rule>> {
        let (time_period, _): (TimePeriod, Summary) = unpack_tree_pack_2_children(tree, Self::get_rule())?;

        Ok(Self(time_period))
    }
}

#[derive(Debug)]
pub struct WorkingDayLog(pub TimePeriod);

impl HasRule for WorkingDayLog {
    type Rule = crate::parser::Rule;
    
    fn get_rule() -> Rule {
        Rule::working_day
    }
}

impl TokenPacker for WorkingDayLog {
    fn pack(tree: SyntaxTree<'_, Self::Rule>) -> Result<Self, PackingError<PackingErrorVariant<Self::Rule>, Self::Rule>> {
        let (time_period, _): (TimePeriod, Option<Summary>) = unpack_tree_pack_maybe_2_children(tree, Self::get_rule())?;

        Ok(Self(time_period))
    }
}

#[derive(Debug)]
pub struct LunchLog(pub TimePeriod);

impl HasRule for LunchLog {
    type Rule = crate::parser::Rule;
    
    fn get_rule() -> Rule {
        Rule::lunch
    }
}

impl TokenPacker for LunchLog {
    fn pack(tree: SyntaxTree<'_, Self::Rule>) -> Result<Self, PackingError<PackingErrorVariant<Self::Rule>, Self::Rule>> {
        let (time_period, _): (TimePeriod, Option<Summary>) = unpack_tree_pack_maybe_2_children(tree, Self::get_rule())?;

        Ok(Self(time_period))
    }
}

#[derive(Debug)]
pub struct BreakLog(pub TimePeriod);

impl HasRule for BreakLog {
    type Rule = crate::parser::Rule;
    
    fn get_rule() -> Rule {
        Rule::r#break
    }
}

impl TokenPacker for BreakLog {
    fn pack(tree: SyntaxTree<'_, Self::Rule>) -> Result<Self, PackingError<PackingErrorVariant<Self::Rule>, Self::Rule>> {
        let (time_period, _): (TimePeriod, Summary) = unpack_tree_pack_2_children(tree, Self::get_rule())?;

        Ok(Self(time_period))
    }
}

#[derive(Debug)]
pub struct LeaveLog(pub TimePeriod);

impl HasRule for LeaveLog {
    type Rule = crate::parser::Rule;
    
    fn get_rule() -> Rule {
        Rule::leave
    }
}

impl TokenPacker for LeaveLog {
    fn pack(tree: SyntaxTree<'_, Self::Rule>) -> Result<Self, PackingError<PackingErrorVariant<Self::Rule>, Self::Rule>> {
        let (time_period, _): (TimePeriod, Option<Summary>) = unpack_tree_pack_maybe_2_children(tree, Self::get_rule())?;

        Ok(Self(time_period))
    }
}

#[derive(Debug)]
pub enum LogEvent {
    Work(WorkLog),
    WorkingDay(WorkingDayLog),
    Lunch(LunchLog),
    Break(BreakLog),
    Leave(LeaveLog),
}

impl HasRule for LogEvent {
    type Rule = crate::parser::Rule;
    
    fn get_rule() -> Rule {
        Rule::log_event
    }
}

impl TokenPacker for LogEvent {
    fn pack(tree: SyntaxTree<'_, Self::Rule>) -> Result<Self, PackingError<PackingErrorVariant<Self::Rule>, Self::Rule>> {
        match get_only_tree_child(tree, Self::get_rule())? {
            t if WorkLog::is_packable(&t) =>
                WorkLog::pack(t).map(Self::Work),
            
            t if WorkingDayLog::is_packable(&t) =>
                WorkingDayLog::pack(t).map(Self::WorkingDay),
            
            t if LunchLog::is_packable(&t) =>
                LunchLog::pack(t).map(Self::Lunch),
            
            t if BreakLog::is_packable(&t) =>
                BreakLog::pack(t).map(Self::Break),

            t if LeaveLog::is_packable(&t) =>
                LeaveLog::pack(t).map(Self::Leave),
            
            t => {
                let expected_rules = vec![
                    WorkLog::get_rule(),
                    WorkingDayLog::get_rule(),
                    LunchLog::get_rule(),
                    BreakLog::get_rule(),
                    LeaveLog::get_rule(),
                ];

                Err(make_wrong_rules_alt_error(t, expected_rules))
            }
        }
    }
}

#[derive(Debug)]
pub struct Log(pub LogEvent);

impl HasRule for Log {
    type Rule = crate::parser::Rule;
    
    fn get_rule() -> Rule {
        Rule::log
    }
}

impl TokenPacker for Log {
    fn pack(tree: SyntaxTree<'_, Self::Rule>) -> Result<Self, PackingError<PackingErrorVariant<Self::Rule>, Self::Rule>> {
        unpack_tree_pack_1_child(tree, Self::get_rule())
            .map(Self)
    }
}

#[derive(Debug)]
pub struct Logs(pub Vec<Log>);

impl HasRule for Logs {
    type Rule = crate::parser::Rule;
    
    fn get_rule() -> Rule {
        Rule::logs
    }
}

impl TokenPacker for Logs {
    fn pack(tree: SyntaxTree<'_, Self::Rule>) -> Result<Self, PackingError<PackingErrorVariant<Self::Rule>, Self::Rule>> {
        unpack_tree_pack_each_child(tree, Self::get_rule())
            .map(Self)
    }
}

#[derive(Debug)]
pub struct Day(pub Weekday, pub Logs);

impl HasRule for Day {
    type Rule = crate::parser::Rule;
    
    fn get_rule() -> Rule {
        Rule::day
    }
}

impl TokenPacker for Day {
    fn pack(tree: SyntaxTree<'_, Self::Rule>) -> Result<Self, PackingError<PackingErrorVariant<Self::Rule>, Self::Rule>> {
        let mut iter = get_tree_children(tree, Self::get_rule())?.into_iter();

        let weekday = pack_next_tree(&mut iter)?;
        let logs = pack_next_tree(&mut iter)?;

        ensure_no_more_trees(iter)?;

        Ok(Self(weekday, logs))
    }
}

#[derive(Debug)]
pub struct Days(pub Vec<Day>);

impl HasRule for Days {
    type Rule = crate::parser::Rule;
    
    fn get_rule() -> Rule {
        Rule::days
    }
}

impl TokenPacker for Days {
    fn pack(tree: SyntaxTree<'_, Self::Rule>) -> Result<Self, PackingError<PackingErrorVariant<Self::Rule>, Self::Rule>> {
        unpack_tree_pack_each_child(tree, Self::get_rule())
            .map(Self)
    }
}

#[derive(Debug)]
pub struct Week(pub Date, pub Days);

impl HasRule for Week {
    type Rule = crate::parser::Rule;
    
    fn get_rule() -> Rule {
        Rule::week
    }
}

impl TokenPacker for Week {
    fn pack(tree: SyntaxTree<'_, Self::Rule>) -> Result<Self, PackingError<PackingErrorVariant<Self::Rule>, Self::Rule>> {
        let mut iter = get_tree_children(tree, Self::get_rule())?.into_iter();

        let date = pack_next_tree(&mut iter)?;
        let days = pack_next_tree(&mut iter)?;

        ensure_no_more_trees(iter)?;

        Ok(Self(date, days))
    }
}

#[derive(Debug)]
pub struct Weeks(pub Vec<Week>);

impl HasRule for Weeks {
    type Rule = crate::parser::Rule;
    
    fn get_rule() -> Rule {
        Rule::weeks
    }
}

impl TokenPacker for Weeks {
    fn pack(tree: SyntaxTree<'_, Self::Rule>) -> Result<Self, PackingError<PackingErrorVariant<Self::Rule>, Self::Rule>> {
        unpack_tree_pack_each_child(tree, Self::get_rule())
            .map(Self)
    }
}

// TODO this should all be generated by an attribute on Body. And should probably have a hygienically generated id
#[derive(Debug)]
pub struct EOI;

impl HasRule for EOI {
    type Rule = crate::parser::Rule;
    
    fn get_rule() -> Rule {
        Rule::EOI
    }
}

impl TokenPacker for EOI {
    fn pack(tree: SyntaxTree<'_, Self::Rule>) -> Result<Self, PackingError<PackingErrorVariant<Self::Rule>, Self::Rule>> {
        get_tree_src_string(tree, Self::get_rule())?;

        Ok(EOI)
    }
}

pub struct Body(pub Weeks);

impl HasRule for Body {
    type Rule = crate::parser::Rule;
    
    fn get_rule() -> Rule {
        Rule::body
    }
}

impl TokenPacker for Body {
    fn pack(tree: SyntaxTree<'_, Self::Rule>) -> Result<Self, PackingError<PackingErrorVariant<Self::Rule>, Self::Rule>> {
        // TODO EOI here should be a special attribute to the macro
        let mut iter = get_tree_children(tree, Self::get_rule())?.into_iter();
        
        let weeks = pack_next_tree(&mut iter)?;
        _ = pack_next_tree::<_, EOI, _>(&mut iter)?;

        ensure_no_more_trees(iter)?;

        Ok(Self(weeks))
    }
}


// #[derive(Packer)]
// #[rule(Rule::some_token)]
// struct SomeToken(Body, Weeks, Day);
