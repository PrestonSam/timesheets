use std::{fmt::Display, iter::once, ops::Not};

use chrono::{DateTime, Local, TimeDelta};
use itertools::Itertools;
use once_cell::sync::Lazy;

use crate::{evaluator::eval::{DayDelta, TotalDelta, WeekDelta}, utils::term_render::{Block, Cell, Column, Segment}};

//
// TODO this should be configuration
static LUNCH_DURATION: Lazy<TimeDelta>
    = Lazy::new(|| TimeDelta::minutes(30));


// This is a bit of a shame.
// Unfortunately I don't own TimeDelta or Display, so I can't write this in a better way.
// Wrapping TimeDelta is possible, but ends up making the whole codebase more clunky for the sake of a couple of lines
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

impl From<&DayDelta> for Cell {
    fn from(DayDelta { weekday, delta, .. }: &DayDelta) -> Self {
        Cell {
            figure: time_delta_to_string(delta),
            comment: weekday.to_owned(),
        }
    }
}

impl From<&WeekDelta> for Block {
    fn from(WeekDelta { starting_date, week_delta, day_deltas }: &WeekDelta) -> Self { 
        let heading = Segment(vec![
            Cell {
                figure: time_delta_to_string(week_delta),
                comment: format!("Week starting {starting_date}"),
            }
        ]);
        let days = day_deltas.iter()
            .map_into()
            .collect();

        Block(vec![
            heading,
            Segment(days),
        ])
    }
}

fn get_deadline_block(total_delta: &TotalDelta) -> Block {
    // TODO record earliest start time for day
    // Use that as the reference point to see if you've worked 7.5 hours yet.
    // Figure out the difference and add that to the time.
    fn get_lunch_if_not_taken(week_deltas: &[WeekDelta]) -> Option<TimeDelta> {
        week_deltas.last()
            ?.day_deltas.last()
            ?.had_lunch.not()
            .then_some(*LUNCH_DURATION)
    }

    fn get_deadline_segment(delta: &TimeDelta, lunch_if_not_taken: Option<&TimeDelta>, message: &str) -> Segment {
        fn to_string(d: DateTime<Local>) -> String {
            d.format("%H:%M").to_string()
        }
        let deadline = Local::now() - *delta;
        let deadline_with_lunch = lunch_if_not_taken.map(|l| to_string(deadline + *l));

        let deadline_cell = Cell {
            figure: to_string(deadline),
            comment: message.into(),
        };

        let lunch_cell = deadline_with_lunch.map(|lunch| 
            Cell {
                figure: lunch,
                comment: format!("{message} + LUNCH"),
            });

        let cells = once(deadline_cell)
            .chain(lunch_cell)
            .collect();
        
        Segment(cells)
    }

    let today_delta = total_delta.week_deltas
        .last()
        .and_then(|w| w.day_deltas.last())
        .map(|day| day.delta)
        .unwrap_or_else(TimeDelta::zero);

    let lunch_if_not_taken = get_lunch_if_not_taken(&total_delta.week_deltas);

    Block(vec![
        get_deadline_segment(&total_delta.total_delta, lunch_if_not_taken.as_ref(), "EARLIEST FINISH TIME"),
        get_deadline_segment(&today_delta, lunch_if_not_taken.as_ref(), "RETAIN CREDIT"),
    ])
}

impl From<&TotalDelta> for Column {
    fn from(value: &TotalDelta) -> Self {
        fn get_credit_str(delta: &TimeDelta) -> &'static str {
            match delta.num_seconds().is_positive() {
                true => "CREDIT",
                false => "DEFICIT",
            }
        }

        let summary = Block(vec![
            Segment(vec![
                Cell {
                    figure: time_delta_to_string(&value.total_delta_excluding_today),
                    comment: format!("TOTAL {} BEFORE TODAY", get_credit_str(&value.total_delta_excluding_today)),
                }
            ]),
            Segment(vec![
                Cell {
                    figure: time_delta_to_string(&value.total_delta),
                    comment: format!("TOTAL {} NOW", get_credit_str(&value.total_delta)),
                }
            ]),
        ]);

        let deadlines = get_deadline_block(value);

        let blocks = value.week_deltas.iter()
            .rev().take(4).rev() // Past four weeks
            .map_into()
            .chain(once(summary))
            .chain(once(deadlines))
            .collect();

        Column(blocks)
    }
}

impl Display for TotalDelta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if 4 < self.week_deltas.len() {
            f.write_str("\n      . . .   Previous weeks truncated\n\n")?;
        }

        f.write_fmt(format_args!("{}", Column::from(self)))
    }
}

