use crate::arg::args::{Arg, BoolArg, DaysOfWeekArg, EntityTypeArg, IntArg};
use crate::command::entity_spec::core::{ArgSlot, ColumnIndexer, PatternIdExt};
use crate::command::entity_spec::event::EventPat;
use crate::core::context::AppContext;
use crate::core::types::{DayOfWeek, EntityType};
use crate::errors::{Error, Result};
use crate::extensions::chrono::WeekdayExt;
use chrono::{Datelike, Local};
use strum::IntoEnumIterator;

// Slots
pub fn id_slot() -> ArgSlot {
    ArgSlot::is_of_arg_type::<IntArg>().with_validator(|arg| match arg {
        Arg::Int(id) if *id > 0 => Ok(()),
        _ => Err(Error::Parse("ID must be greater than 0.".into())),
    })
}

pub fn entity_slot(expected: EntityType) -> ArgSlot {
    ArgSlot::is_of_arg_type::<EntityTypeArg>().with_validator(move |a| match a {
        Arg::EntityType(t) if *t == expected => Ok(()),
        Arg::EntityType(got) => Err(Error::Parse(format!(
            "Wrong entity type: expected {}, got {}",
            expected, got
        ))),
        _ => unreachable!("kind already checked by ArgMatcher"),
    })
}

// Validators

pub fn card_id_validator() -> Box<dyn Fn(&Arg, &AppContext) -> Result<()> + 'static> {
    Box::new(|arg, ctx| {
        if let Arg::CardColorId(id) = arg {
            if !ctx.cards.exists_including_staged(*id) {
                return Err(Error::Parse(format!("Card id {} does not exist.", id)));
            }
        }
        Ok(())
    })
}

pub fn daily_hour_range_validator() -> Box<dyn Fn(&Arg, &AppContext) -> Result<()> + 'static> {
    Box::new(|arg, ctx| {
        if let Arg::TimeRange(range) = arg {
            let daily_hours_range = ctx.config.range();
            if range.start < daily_hours_range.start || range.end > daily_hours_range.end {
                return Err(Error::Parse(format!(
                    "Event falls outside of daily hours range {} from config",
                    daily_hours_range
                )));
            }
        }
        Ok(())
    })
}

pub fn task_start_date_validator() -> Box<dyn Fn(&Arg, &AppContext) -> Result<()> + 'static> {
    Box::new(|arg, ctx| {
        if let Arg::Date(d) = arg {
            if let Some(start) = ctx.config.schedule_start_date() {
                if d.0 < *start {
                    return Err(Error::Parse(format!(
                        "Task due date {} cannot be before schedule start date {}.",
                        d.0, start
                    )));
                }
            }
        }
        Ok(())
    })
}

pub fn validate_event_recurring_days(args: &[Arg], pid: EventPat) -> Result<()> {
    let pattern = pid.pattern();
    let mut ix = ColumnIndexer::new(args, &pattern);

    match pid {
        EventPat::Base => {
            let recurring = ix.next::<BoolArg>().0;
            if let Some(days) = ix.advance_until_arg::<DaysOfWeekArg>() {
                if !recurring && days.len() != 1 {
                    return Err(Error::Parse(
                        "Non-recurring events must have exactly one day.".into(),
                    ));
                }
            }
        }
        EventPat::EntityFirst => {
            let recurring = ix.advance_times(2).next::<BoolArg>().0;
            if let Some(days) = ix.advance_until_arg::<DaysOfWeekArg>() {
                if !recurring && days.len() != 1 {
                    return Err(Error::Parse(
                        "Non-recurring events must have exactly one day.".into(),
                    ));
                }
            }
        }
        EventPat::EntityId => {}
    }

    Ok(())
}

pub fn default_days_for(recurring: bool) -> Vec<DayOfWeek> {
    if recurring {
        DayOfWeek::iter().collect()
    } else {
        vec![Local::now().weekday().to_day_of_week()]
    }
}
