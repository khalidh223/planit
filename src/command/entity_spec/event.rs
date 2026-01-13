use crate::arg::args::{
    Arg, AtSymbolArg, BoolArg, CardColorIdArg, DaysOfWeekArg, NameArg, TimeRangeArg,
};
use crate::command::entity_spec::common::{
    card_id_validator, daily_hour_range_validator, default_days_for, entity_slot, id_slot,
    validate_event_recurring_days,
};
use crate::command::entity_spec::core::{
    ArgPattern, ArgSchema, ArgSlot, ArgValidator, ColumnIndexer, EntityBuilder, EntitySpec,
    PatternIdExt,
};
use crate::core::context::AppContext;
use crate::core::models::Event;
use crate::core::types::{EntityActionType, EntityType};
use crate::errors::{Error, Result};
use std::fmt;

pub struct EventArgSchema;

impl EventArgSchema {
    fn pattern_base() -> ArgPattern {
        vec![
            ArgSlot::is_of_arg_type::<BoolArg>(),
            ArgSlot::is_of_arg_type::<NameArg>(),
            ArgSlot::is_of_arg_type::<CardColorIdArg>()
                .optional()
                .with_validator_ctx(card_id_validator()),
            ArgSlot::is_of_arg_type::<AtSymbolArg>(),
            ArgSlot::is_of_arg_type::<DaysOfWeekArg>().optional(),
            ArgSlot::is_of_arg_type::<TimeRangeArg>()
                .with_validator_ctx(daily_hour_range_validator()),
        ]
    }

    fn pattern_entity_id() -> ArgPattern {
        vec![entity_slot(EntityType::Event), id_slot()]
    }

    fn pattern_entity_first() -> ArgPattern {
        let mut v = Self::pattern_entity_id();
        v.extend(Self::pattern_base());
        v
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventPat {
    Base,
    EntityFirst,
    EntityId,
}

impl EventPat {
    const fn usage(self) -> &'static str {
        match self {
            EventPat::Base => {
                r#"event <bool> "<name>" [cardId] @ [days of week] <time range>
Required:
  bool         - (bool)      Whether the event is recurring (true/false)
  name         - (string)    Name of event, wrapped in single or double quotes
  time range   - (TimeRange) Start & end time of the event. Run 'time -h' to see valid time formats for start/end.
Optional:
  cardId       - (integer)   Id referencing a Card for its tag and color. Must prefix with '+C'
  days of week - (DayOfWeek) Comma separated list of one or more days of the week"#
            }

            EventPat::EntityFirst => {
                r#"event <id> <bool> "<name>" [cardId] @ <days of week> <time range>
Required:
  id           - (int)       id of event
  bool         - (bool)      Whether the event is recurring (true/false)
  name         - (string)    Name of event, wrapped in single or double quotes
  time range   - (TimeRange) Start & end time of the event. Run 'time -h' to see valid time formats for start/end
Optional:
  cardId       - (integer) Id referencing a Card for its tag and color. Must prefix with '+C'
  days of week - (DayOfWeek) Comma separated list of one or more days of the week"#
            }

            EventPat::EntityId => {
                r#"event <id>
Required:
  id    - (int)    id of event"#
            }
        }
    }
}

impl fmt::Display for EventPat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.usage())
    }
}

impl PatternIdExt for EventPat {
    fn pattern(&self) -> ArgPattern {
        match self {
            EventPat::Base => EventArgSchema::pattern_base(),
            EventPat::EntityFirst => EventArgSchema::pattern_entity_first(),
            EventPat::EntityId => EventArgSchema::pattern_entity_id(),
        }
    }
}

impl ArgSchema for EventArgSchema {
    type PatternId = EventPat;

    fn patterns_for(&self, action: EntityActionType) -> Vec<EventPat> {
        match action {
            EntityActionType::Add => vec![EventPat::Base],
            EntityActionType::Modify => vec![EventPat::EntityFirst],
            EntityActionType::Delete => vec![EventPat::EntityId],
        }
    }
}

pub struct EventArgValidator;
impl ArgValidator for EventArgValidator {
    type PatternId = EventPat;

    fn validate(&self, args: &[Arg], _: EntityActionType, pat_id: Self::PatternId) -> Result<()> {
        match pat_id {
            EventPat::Base => validate_event_recurring_days(args, pat_id),
            EventPat::EntityFirst => validate_event_recurring_days(args, pat_id),
            EventPat::EntityId => Ok(()),
        }
    }
}

pub struct EventBuilder;
impl EntityBuilder<Event> for EventBuilder {
    type PatternId = EventPat;

    fn create(&self, args: &[Arg], pat_id: EventPat) -> Result<Event> {
        match pat_id {
            EventPat::Base => {
                let pattern = pat_id.pattern();
                let mut ix = ColumnIndexer::new(args, &pattern);
                let recurring = ix.next::<BoolArg>().0;
                Ok(Event::new(
                    recurring,
                    ix.next::<NameArg>().clone(),
                    ix.next_opt::<CardColorIdArg>(),
                    ix.advance()
                        .next_opt::<DaysOfWeekArg>()
                        .map(|v| v.clone())
                        .unwrap_or_else(|| default_days_for(recurring)),
                    ix.next::<TimeRangeArg>().clone(),
                ))
            }
            _ => Err(Error::Parse(
                "No valid ADD pattern matched for event.".into(),
            )),
        }
    }

    fn modify<'a>(
        &self,
        existing: &'a mut Event,
        args: &[Arg],
        pat_id: EventPat,
    ) -> Result<&'a Event> {
        match pat_id {
            EventPat::EntityFirst => {
                let pattern = pat_id.pattern();
                let mut ix = ColumnIndexer::new(args, &pattern);
                let recurring = ix.advance_times(2).next::<BoolArg>().0;
                existing.modify(
                    recurring,
                    ix.next::<NameArg>().clone(),
                    ix.next_opt::<CardColorIdArg>(),
                    ix.advance()
                        .next_opt::<DaysOfWeekArg>()
                        .map(|v| v.clone())
                        .unwrap_or_else(|| default_days_for(recurring)),
                    ix.next::<TimeRangeArg>().clone(),
                );
                Ok(&*existing)
            }
            _ => Err(Error::Parse(
                "No valid MODIFY pattern matched for event.".into(),
            )),
        }
    }
}

pub struct EventSpec {
    schema: EventArgSchema,
    validator: EventArgValidator,
    builder: EventBuilder,
}

impl EventSpec {
    pub fn new() -> Self {
        Self {
            schema: EventArgSchema,
            validator: EventArgValidator,
            builder: EventBuilder,
        }
    }
}

impl EntitySpec<Event> for EventSpec {
    type PatternId = EventPat;

    fn arg_schema(&self) -> &dyn ArgSchema<PatternId = EventPat> {
        &self.schema
    }
    fn arg_validator(&self) -> &dyn ArgValidator<PatternId = EventPat> {
        &self.validator
    }
    fn entity_builder(&self) -> &dyn EntityBuilder<Event, PatternId = EventPat> {
        &self.builder
    }

    fn get_mut<'a>(&self, ctx: &'a mut AppContext, id: i32) -> Result<&'a mut Event> {
        ctx.events.get_mut(id)
    }
}
