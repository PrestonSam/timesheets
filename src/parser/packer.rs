use chrono::NaiveTime;
use lang_packer::Packer;

use crate::parser::Rule;

#[derive(Debug, Packer)]
#[packer(rule = Rule::DATE)]
pub struct Date(pub String);

#[derive(Debug, Packer)]
#[packer(rule = Rule::DAY_NAME)]
pub struct DayName(pub String);

#[derive(Debug, Packer)]
#[packer(rule = Rule::SUMMARY)]
pub struct Summary(String);

#[derive(Debug, Clone, Packer)]
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

#[derive(Debug, Clone, Packer)]
#[packer(rule = Rule::NOW)]
pub struct Now;

#[derive(Debug, Clone, Packer)]
#[packer(rule = Rule::time_range_end)]
pub enum TimeRangeEnd {
    Now(Now),
    Time(Time),
}

#[derive(Debug, Clone, Packer)]
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
pub struct WorkLog(pub TimePeriod, pub Summary);

#[derive(Debug, Packer)]
#[packer(rule = Rule::working_day)]
pub struct WorkingDayLog(pub TimePeriod, pub Option<Summary>);

#[derive(Debug, Packer)]
#[packer(rule = Rule::lunch)]
pub struct LunchLog(pub TimePeriod, pub Option<Summary>);

#[derive(Debug, Packer)]
#[packer(rule = Rule::r#break)]
pub struct BreakLog(pub TimePeriod, pub Summary);

#[derive(Debug, Packer)]
#[packer(rule = Rule::leave)]
pub struct LeaveLog(pub TimePeriod, pub Option<Summary>);

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
#[packer(rule = Rule::day)]
pub struct Day(pub DayName, pub Vec<Log>);

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





impl std::fmt::Display for Period {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Period::Minutes(Minutes(Number(minutes))) =>
                f.write_fmt(format_args!("{minutes}m")),

            Period::HoursMinutes(HoursMinutes(Hours(Number(hours)), Minutes(Number(minutes)))) =>
                f.write_fmt(format_args!("{hours}h {minutes}m")),
        }
    }
}

impl std::fmt::Display for Time {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.0.format("%H:%M")))
    }
}

impl std::fmt::Display for TimeRangeEnd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TimeRangeEnd::Now(_) => f.write_str("NOW"),
            TimeRangeEnd::Time(time) => f.write_fmt(format_args!("{time}")),
        }
    }
}

impl std::fmt::Display for TimeRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let TimeRange(time, time_range_end) = self;

        f.write_fmt(format_args!("{time} - {time_range_end}"))
    }
}

impl std::fmt::Display for TimePeriod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TimePeriod::Period(period) => f.write_fmt(format_args!("{period}")),
            TimePeriod::TimeRange(time_range) => f.write_fmt(format_args!("{time_range}")),
        }
    }
}

impl std::fmt::Display for WorkingDayLog {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let WorkingDayLog(period, maybe_summary) = self;

        f.write_fmt(format_args!("    WORKING DAY {period}"))?;

        if let Some(Summary(summary)) = maybe_summary {
            f.write_fmt(format_args!(" | {summary}"))?;
        }

        Ok(())
   }
}
