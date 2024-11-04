use std::fmt::Display;

use chrono::{Local, TimeDelta};
use once_cell::sync::Lazy;

use crate::packer::{
    BreakLog, Date, Day, Days, Hours, HoursMinutes, LeaveLog, Log, LogEvent, Logs, LunchLog, Minutes, Number,
    Period, Time, TimePeriod, TimeRange, TimeRangeEnd, Week, Weekday, Weeks, WorkLog, WorkingDayLog
};

#[derive(Debug)]
pub enum EvalError { // TODO
}

fn delta_to_str(delta: &TimeDelta) -> String {
    let delta_in_minutes = delta.num_minutes().abs();
    let (hours, minutes) = (delta_in_minutes / 60, delta_in_minutes % 60);
    let sign = match delta.num_minutes().is_negative() {
        true => '-',
        false => '+',
    };

    let time_str = match hours {
        0 => format!("{minutes}m", ),
        _ => format!("{hours}:{minutes:0>2}"),
    };

    format!("{sign}{time_str:<4}")
}

pub struct DayDelta {
    pub weekday: String,
    pub delta: TimeDelta,
    pub had_lunch: bool,
}

pub struct WeekDelta {
    pub starting_date: String,
    pub week_delta: TimeDelta,
    pub day_deltas: Vec<DayDelta>,
}

impl Display for WeekDelta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let starting_date = self.starting_date.as_str();
        let week_surplus_str = delta_to_str(&self.week_delta);

        f.write_fmt(format_args!( 
"    ┌───────┐
    │ {week_surplus_str:<5} │ Week starting {starting_date}
    ├───────┤
"))?;
        
        for DayDelta { weekday, delta, .. } in self.day_deltas.iter() {
            f.write_fmt(format_args!("    │ {:<5} │ {weekday}\n", delta_to_str(&delta)))?;
        }

        f.write_str("    └───────┘\n")?;

        Ok(())
    }
}

pub struct TotalDelta {
    pub total_delta: TimeDelta,
    pub total_delta_excluding_today: TimeDelta,
    pub week_deltas: Vec<WeekDelta>,
}

impl Display for TotalDelta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if 4 < self.week_deltas.len() {
            f.write_str("\n      ...     Previous weeks truncated\n\n")?;
        }

        self.week_deltas
            .iter()
            .rev().take(4).rev()
            .map(|w| f.write_fmt(format_args!("{w}")))
            .collect::<Result<(), _>>()?;

        fn get_credit_str(delta: &TimeDelta) -> &'static str {
            match delta.num_seconds().is_positive() {
                true => "CREDIT",
                false => "DEFICIT",
            }
        }

        f.write_str("    ┌───────┐\n")?;
        f.write_fmt(format_args!(
            "    │ {:<5} │ TOTAL {} BEFORE TODAY\n",
            delta_to_str(&self.total_delta_excluding_today),
            get_credit_str(&self.total_delta_excluding_today)
        ))?;
        f.write_fmt(format_args!(
            "    │ {:<5} │ TOTAL {} NOW\n",
            delta_to_str(&self.total_delta),
            get_credit_str(&self.total_delta)
        ))?;
        f.write_str("    └───────┘\n")?;

        Ok(())
    }
}

fn eval_period(period: Period) -> Result<TimeDelta, EvalError> {
    match period {
        Period::HoursMinutes(HoursMinutes(Hours(Number(hours)), Minutes(Number(minutes)))) =>
            Ok(TimeDelta::hours(hours as i64) + TimeDelta::minutes(minutes as i64)),

        Period::Minutes(Minutes(Number(minutes))) =>
            Ok(TimeDelta::minutes(minutes as i64)),
    }
}

fn eval_time_range(time_range: TimeRange) -> Result<TimeDelta, EvalError> {
    let TimeRange(Time(start), end) = time_range;

    let end = match end {
        TimeRangeEnd::Time(Time(end)) =>
            end,

        TimeRangeEnd::Now(_) =>
            Local::now().time(),
    };

    Ok(end - start)
}

fn eval_time_period(time_period: TimePeriod) -> Result<TimeDelta, EvalError> {
    match time_period {
        TimePeriod::Period(period) =>
            eval_period(period),

        TimePeriod::TimeRange(time_range) =>
            eval_time_range(time_range),
    }
}

fn eval_log(log: Log) -> Result<TimeDelta, EvalError> {
    let Log(event) = log;

    let period = match event {
        LogEvent::Break(BreakLog(period)) => -(eval_time_period(period)?),
        LogEvent::Leave(LeaveLog(period)) => eval_time_period(period)?,
        LogEvent::Lunch(LunchLog(period)) => -(eval_time_period(period)?),
        LogEvent::Work(WorkLog(period)) => eval_time_period(period)?,
        LogEvent::WorkingDay(WorkingDayLog(period)) => eval_time_period(period)?,
    };

    Ok(period)
}

static WORKING_DAY: Lazy<TimeDelta>
    = Lazy::new(|| TimeDelta::hours(7) + TimeDelta::minutes(30));

fn eval_day(day: Day) -> Result<DayDelta, EvalError> {
    let Day(Weekday(weekday), Logs(logs)) = day;

    let had_lunch = logs.iter()
        .any(|log| matches!(log, Log(LogEvent::Lunch(_))));

    logs.into_iter()
        .map(eval_log)
        .sum::<Result<TimeDelta, _>>()
        .map(|delta| DayDelta {  had_lunch, weekday, delta: delta - *WORKING_DAY })
}

fn eval_week(week: Week) -> Result<WeekDelta, EvalError> {
    let Week(Date(starting_date), Days(days)) = week;

    days.into_iter()
        .map(eval_day)
        .collect::<Result<Vec<_>, EvalError>>()
        .map(|day_deltas| WeekDelta {
            starting_date,
            week_delta: day_deltas.iter()
                .map(|d| d.delta)
                .sum(),
            day_deltas
        })
}

pub fn evaluate_timesheets(weeks: Weeks) -> Result<TotalDelta, EvalError> {
    let Weeks(weeks) = weeks;

    weeks.into_iter()
        .map(eval_week)
        .collect::<Result<Vec<_>, EvalError>>()
        .map(|week_deltas| {
            let total_delta = week_deltas.iter()
                    .map(|w| w.week_delta)
                    .sum();

            // TODO add check to make sure this is actually today. Currently assumes that today is already in the process of being logged
            let today_delta = week_deltas
                .last()
                .and_then(|w| w.day_deltas.last())
                .map(|day| day.delta)
                .unwrap_or_else(|| TimeDelta::zero());

            TotalDelta {
                total_delta, 
                total_delta_excluding_today: total_delta - today_delta,
                week_deltas,
            }
        })
}
