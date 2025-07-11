use std::fmt::Display;

use chrono::{Local, TimeDelta};
use once_cell::sync::Lazy;

use crate::parser::{
    BreakLog, Date, Day, DayName, Days, Hours, HoursMinutes, LeaveLog, Log, LogEvent, LunchLog, Minutes,
    Number, Period, Time, TimePeriod, TimeRange, TimeRangeEnd, Week, Weeks, WorkLog, WorkingDayLog
};

fn get_credit_str(delta: &TimeDelta) -> &'static str {
    match delta.num_seconds().is_positive() {
        true => "CREDIT",
        false => "DEFICIT",
    }
}

fn time_delta_to_string(delta: &TimeDelta) -> String {
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
        f.write_fmt(format_args!( 
"    ┌───────┐
    │ {:<5} │ Week starting {}
    ├───────┤
",
            time_delta_to_string(&self.week_delta),
            self.starting_date
        ))?;
        
        for DayDelta { weekday, delta, .. } in self.day_deltas.iter() {
            f.write_fmt(format_args!("    │ {:<5} │ {weekday}\n", time_delta_to_string(delta)))?;
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
            f.write_str("\n      . . .   Previous weeks truncated\n\n")?;
        }

        self.week_deltas
            .iter()
            .rev().take(4).rev()
            .map(|w| w.fmt(f))
            .collect::<std::fmt::Result>()?;

        f.write_fmt(format_args!(
"    ┌───────┐
    │ {:<5} │ TOTAL {} BEFORE TODAY
    │ {:<5} │ TOTAL {} NOW
    └───────┘
",
            time_delta_to_string(&self.total_delta_excluding_today),
            get_credit_str(&self.total_delta_excluding_today),
            time_delta_to_string(&self.total_delta),
            get_credit_str(&self.total_delta),
        ))
    }
}

impl Period {
    fn evaluate(self) -> TimeDelta {
        match self {
            Period::HoursMinutes(HoursMinutes(Hours(Number(hours)), Minutes(Number(minutes)))) =>
                TimeDelta::hours(hours) + TimeDelta::minutes(minutes),

            Period::Minutes(Minutes(Number(minutes))) =>
                TimeDelta::minutes(minutes),
        }
    }
}

impl TimeRange {
    fn evaluate(self) -> TimeDelta {
        let TimeRange(Time(start), end) = self;

        let end = match end {
            TimeRangeEnd::Time(Time(end)) =>
                end,
                // cmp::min(end, now), // Should be capped at 'now' if evaluating the current day
                // Sadly that information is out of scope at the moment

            TimeRangeEnd::Now(_) =>
                Local::now().time(),
        };

        end - start
    }
}

impl TimePeriod {
    fn evaluate(self) -> TimeDelta {
        match self {
            TimePeriod::Period(period) => period.evaluate(),
            TimePeriod::TimeRange(time_range) => time_range.evaluate(),
        }
    }
}

impl Log {
    fn evaluate(self) -> TimeDelta {
        let Log(event) = self;

        match event {
            LogEvent::Break(BreakLog(period)) => -period.evaluate(),
            LogEvent::Leave(LeaveLog(period)) => period.evaluate(),
            LogEvent::Lunch(LunchLog(period)) => -period.evaluate(),
            LogEvent::Work(WorkLog(period)) => period.evaluate(),
            LogEvent::WorkingDay(WorkingDayLog(period)) => period.evaluate(),
        }
    }
}

static WORKING_DAY: Lazy<TimeDelta>
    = Lazy::new(|| TimeDelta::hours(7) + TimeDelta::minutes(30));

impl Day {
    fn evaluate(self) -> DayDelta {
        let Day(DayName(weekday), logs) = self;

        let had_lunch = logs.iter()
            .any(|log| matches!(log, Log(LogEvent::Lunch(_))));

        let delta = logs.into_iter()
            .map(Log::evaluate)
            .sum::<TimeDelta>();

        DayDelta {
            had_lunch,
            weekday,
            delta: delta - *WORKING_DAY
        }
    }
}

fn eval_week(week: Week) -> WeekDelta {
    let Week(Date(starting_date), Days(days)) = week;

    let day_deltas = days.into_iter()
        .map(Day::evaluate)
        .collect::<Vec<_>>();

    WeekDelta {
        starting_date,
        week_delta: day_deltas.iter()
            .map(|d| d.delta)
            .sum(),
        day_deltas
    }
}

pub fn evaluate_timesheets(weeks: Weeks) -> TotalDelta {
    let Weeks(weeks) = weeks;

    let week_deltas = weeks.into_iter()
        .map(eval_week)
        .collect::<Vec<_>>();

    let total_delta = week_deltas.iter()
        .map(|w| w.week_delta)
        .sum();

    // TODO add check to make sure this is actually today.
    // Currently assumes that today is already in the process of being logged
    let today_delta = week_deltas.last()
        .and_then(|w| w.day_deltas.last())
        .map(|day| day.delta)
        .unwrap_or_else(TimeDelta::zero);

    TotalDelta {
        total_delta, 
        total_delta_excluding_today: total_delta - today_delta,
        week_deltas,
    }
}
