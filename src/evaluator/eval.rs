use chrono::{Local, TimeDelta};
use once_cell::sync::Lazy;

use crate::parser::*;

// TODO this should be configuration
static WORKING_DAY_DURATION: Lazy<TimeDelta> 
    = Lazy::new(|| TimeDelta::hours(8) + TimeDelta::minutes(00));

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

pub struct TotalDelta {
    pub total_delta: TimeDelta,
    pub total_delta_excluding_today: TimeDelta,
    pub week_deltas: Vec<WeekDelta>,
}


impl Period {
    fn evaluate(self) -> TimeDelta {
        match self {
            Period::HoursMinutes(HoursMinutes(hours, Some(minutes))) =>
                TimeDelta::hours(*hours) + TimeDelta::minutes(*minutes),

            Period::HoursMinutes(HoursMinutes(hours, None)) =>
                TimeDelta::hours(*hours),

            Period::Minutes(minutes) =>
                TimeDelta::minutes(*minutes),
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
            delta: delta - *WORKING_DAY_DURATION
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
