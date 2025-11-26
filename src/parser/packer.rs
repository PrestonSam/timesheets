use std::ops::Deref;

use chrono::NaiveTime;
use lang_packer::Packer;

use crate::parser::Rule;

#[derive(Debug, Packer)]
#[packer(rule = Rule::DATE)]
pub struct Date(pub String);

#[derive(Debug, Packer)]
#[packer(rule = Rule::DAY_NAME)]
pub struct DayName(pub String);

#[derive(Debug, Clone, Packer)]
#[packer(rule = Rule::TIME)]
pub struct Time(pub NaiveTime);

#[derive(Debug, Packer)]
#[packer(rule = Rule::numbers)]
pub struct Number(pub i64);

#[derive(Debug, Packer)]
#[packer(rule = Rule::PERIOD_MINUTES)]
pub struct Minutes(pub Number);

#[derive(Debug, Packer)]
#[packer(rule = Rule::PERIOD_HOURS)]
pub struct Hours(pub Number);

#[derive(Debug, Packer)]
#[packer(rule = Rule::period_hours_minutes)]
pub struct HoursMinutes(pub Hours, pub Option<Minutes>);

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



impl Deref for Hours {
    type Target = i64;

    fn deref(&self) -> &Self::Target {
        &self.0.0
    }
}

impl Deref for Minutes {
    type Target = i64;

    fn deref(&self) -> &Self::Target {
        &self.0.0
    }
} 


impl std::fmt::Display for Period {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Period::Minutes(Minutes(Number(minutes))) =>
                f.write_fmt(format_args!("{minutes}m")),

            Period::HoursMinutes(HoursMinutes(Hours(Number(hours)), None)) =>
                f.write_fmt(format_args!("{hours}h")),

            Period::HoursMinutes(HoursMinutes(Hours(Number(hours)), Some(Minutes(Number(minutes))))) =>
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
        let WorkingDayLog(period) = self;

        f.write_fmt(format_args!("    WORKING DAY {period}"))?;

        Ok(())
   }
}
