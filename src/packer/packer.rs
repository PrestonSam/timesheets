use chrono::NaiveTime;
use lang_packer::Packer;
use token_packer::pack_trees::HasRule;
use token_packer::generic_model::PackingResult;

use crate::parser::Rule;

#[derive(Debug, Packer)]
#[packer(rule = Rule::DATE)]
pub struct Date(pub String);

#[derive(Debug, Packer)]
#[packer(rule = Rule::WEEKDAY)]
pub struct Weekday(pub String);

#[derive(Debug, Packer)]
#[packer(rule = Rule::TIME)]
pub struct Time(pub NaiveTime);

#[derive(Debug, Packer)]
#[packer(rule = Rule::numbers)]
pub struct Number(pub usize);

#[derive(Debug, Packer)]
#[packer(rule = Rule::PERIOD_MINUTES)]
pub struct Minutes(pub Number);

#[derive(Debug, Packer)]
#[packer(rule = Rule::PERIOD_HOURS)]
pub struct Hours(pub Number);

#[derive(Debug, Packer)]
#[packer(rule = Rule::period_hours_minutes)]
pub struct HoursMinutes(pub Hours, pub Minutes);

#[derive(Debug, Packer)]
#[packer(rule = Rule::PERIOD)]
pub enum Period {
    Minutes(Minutes),
    HoursMinutes(HoursMinutes),
}

#[derive(Debug, Packer)]
#[packer(rule = Rule::NOW)]
pub struct Now;

#[derive(Debug, Packer)]
#[packer(rule = Rule::time_range_end)]
pub enum TimeRangeEnd {
    Now(Now),
    Time(Time),
}

#[derive(Debug, Packer)]
#[packer(rule = Rule::time_range)]
pub struct TimeRange(pub Time, pub TimeRangeEnd);

#[derive(Debug, Packer)]
#[packer(rule = Rule::time_period)]
pub enum TimePeriod {
    Period(Period),
    TimeRange(TimeRange),
}

#[derive(Debug, Packer)]
#[packer(rule = Rule::work)]
pub struct WorkLog(pub TimePeriod);

#[derive(Debug, Packer)]
#[packer(rule = Rule::working_day)]
pub struct WorkingDayLog(pub TimePeriod);

#[derive(Debug, Packer)]
#[packer(rule = Rule::lunch)]
pub struct LunchLog(pub TimePeriod);

#[derive(Debug, Packer)]
#[packer(rule = Rule::r#break)]
pub struct BreakLog(pub TimePeriod);

#[derive(Debug, Packer)]
#[packer(rule = Rule::leave)]
pub struct LeaveLog(pub TimePeriod);

#[derive(Debug, Packer)]
#[packer(rule = Rule::log_event)]
pub enum LogEvent {
    Work(WorkLog),
    WorkingDay(WorkingDayLog),
    Lunch(LunchLog),
    Break(BreakLog),
    Leave(LeaveLog),
}

#[derive(Debug, Packer)]
#[packer(rule = Rule::log)]
pub struct Log(pub LogEvent);

#[derive(Debug, Packer)]
#[packer(rule = Rule::logs)]
pub struct Logs(pub Vec<Log>);

#[derive(Debug, Packer)]
#[packer(rule = Rule::day)]
pub struct Day(pub Weekday, pub Logs);

#[derive(Debug, Packer)]
#[packer(rule = Rule::days)]
pub struct Days(pub Vec<Day>);

#[derive(Debug, Packer)]
#[packer(rule = Rule::week)]
pub struct Week(pub Date, pub Days);

#[derive(Debug, Packer)]
#[packer(rule = Rule::weeks)]
pub struct Weeks(pub Vec<Week>);

#[derive(Debug, Packer)]
#[packer(rule = Rule::EOI)]
pub struct EOI;

#[derive(Debug, Packer)]
#[packer(rule = Rule::body)]
pub struct Body(pub Weeks, pub EOI);
