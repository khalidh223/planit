use super::{
    card::{CardBuilder, CardPat, CardSpec},
    common::{card_id_validator, default_days_for, validate_event_recurring_days},
    core::{ArgSlot, ColumnIndexer, EntityBuilder, EntitySpec, PatternIdExt, SlotMatch},
    event::{EventBuilder, EventPat, EventSpec},
    task::{TaskBuilder, TaskPat, TaskSpec},
};
use crate::arg::args::Arg;
use crate::arg::args::{AtSymbolArg, CardColorIdArg, DateArg, IntArg, NameArg};
use crate::core::context::AppContext;
use crate::core::models::Card;
use crate::core::types::{
    Bool, CardColor, Date, DayOfWeek, EntityActionType, EntityType, TimeRange,
};
use crate::errors::Error;
use strum::IntoEnumIterator;

fn ctx() -> AppContext {
    AppContext::new()
}

fn future_date() -> Date {
    Date::try_from_str("2099-01-01").unwrap()
}

fn sample_time_range() -> TimeRange {
    TimeRange::try_from_str("8AM-9AM").unwrap()
}

// ---------- core helpers ----------
#[test]
fn arg_slot_classify_runs_validator() {
    let slot =
        ArgSlot::is_of_arg_type::<crate::arg::args::IntArg>().with_validator(|arg| match arg {
            Arg::Int(v) if *v > 0 => Ok(()),
            _ => Err(Error::Parse("must be positive".into())),
        });

    let ctx = ctx();
    match slot.classify(&Arg::Int(5), &ctx) {
        SlotMatch::Match => {}
        other => panic!("expected match, got {other:?}"),
    }

    match slot.classify(&Arg::Int(0), &ctx) {
        SlotMatch::ValidatorFail(_) => {}
        other => panic!("expected validator fail, got {other:?}"),
    }

    match slot.classify(&Arg::Name("x".into()), &ctx) {
        SlotMatch::KindMismatch(_) => {}
        other => panic!("expected kind mismatch, got {other:?}"),
    }
}

#[test]
fn column_indexer_skips_optional_slots() {
    let pattern = TaskPat::Base.pattern();
    let date = future_date();
    let args = vec![
        Arg::Name("Task".into()),
        Arg::Int(2),
        Arg::AtSymbol,
        Arg::Date(date.clone()),
    ];

    let mut ix = ColumnIndexer::new(&args, &pattern);
    assert_eq!(ix.next::<NameArg>(), "Task");
    assert_eq!(ix.next::<IntArg>(), 2);
    assert!(ix.next_opt::<CardColorIdArg>().is_none()); // optional skipped
    ix.next::<AtSymbolArg>();
    assert_eq!(*ix.next::<DateArg>(), date);
}

#[test]
fn column_indexer_advance_skips_first_slot() {
    let pattern = vec![
        ArgSlot::is_of_arg_type::<NameArg>(),
        ArgSlot::is_of_arg_type::<IntArg>(),
    ];
    let args = vec![Arg::Name("Alpha".into()), Arg::Int(7)];

    let mut ix = ColumnIndexer::new(&args, &pattern);
    ix.advance();
    assert_eq!(ix.next::<IntArg>(), 7);
}

#[test]
fn column_indexer_advance_times_skips_multiple_slots() {
    let pattern = vec![
        ArgSlot::is_of_arg_type::<NameArg>(),
        ArgSlot::is_of_arg_type::<IntArg>(),
        ArgSlot::is_of_arg_type::<AtSymbolArg>(),
        ArgSlot::is_of_arg_type::<DateArg>(),
    ];
    let date = future_date();
    let args = vec![
        Arg::Name("Alpha".into()),
        Arg::Int(3),
        Arg::AtSymbol,
        Arg::Date(date.clone()),
    ];

    let mut ix = ColumnIndexer::new(&args, &pattern);
    ix.advance_times(2);
    ix.next::<AtSymbolArg>();
    assert_eq!(*ix.next::<DateArg>(), date);
}

#[test]
fn column_indexer_advance_until_arg_finds_later_slot() {
    let pattern = vec![
        ArgSlot::is_of_arg_type::<NameArg>(),
        ArgSlot::is_of_arg_type::<CardColorIdArg>().optional(),
        ArgSlot::is_of_arg_type::<IntArg>(),
    ];
    let args = vec![Arg::Name("Alpha".into()), Arg::Int(2)];

    let mut ix = ColumnIndexer::new(&args, &pattern);
    let value = ix.advance_until_arg::<IntArg>().unwrap();
    assert_eq!(value, 2);
}

#[test]
fn column_indexer_advance_until_arg_returns_none_when_missing() {
    let pattern = vec![
        ArgSlot::is_of_arg_type::<NameArg>(),
        ArgSlot::is_of_arg_type::<IntArg>(),
    ];
    let args = vec![Arg::Name("Alpha".into())];

    let mut ix = ColumnIndexer::new(&args, &pattern);
    let value = ix.advance_until_arg::<DateArg>();
    assert!(value.is_none());
}

#[test]
fn column_indexer_next_opt_returns_none_for_missing_required() {
    let pattern = vec![
        ArgSlot::is_of_arg_type::<NameArg>(),
        ArgSlot::is_of_arg_type::<IntArg>(),
    ];
    let args = vec![Arg::Name("Alpha".into())];

    let mut ix = ColumnIndexer::new(&args, &pattern);
    assert_eq!(ix.next::<NameArg>(), "Alpha");
    let value = ix.next_opt::<IntArg>();
    assert!(value.is_none());
}

#[test]
#[should_panic(expected = "pattern matched; required slot must be present")]
fn column_indexer_next_panics_on_missing_required() {
    let pattern = vec![
        ArgSlot::is_of_arg_type::<NameArg>(),
        ArgSlot::is_of_arg_type::<IntArg>(),
    ];
    let args = vec![Arg::Name("Alpha".into())];

    let mut ix = ColumnIndexer::new(&args, &pattern);
    ix.next::<NameArg>();
    let _ = ix.next::<IntArg>();
}

// ---------- common helpers ----------
#[test]
fn card_id_validator_checks_repository() {
    let mut ctx = ctx();
    let v = card_id_validator();
    let missing = v(&Arg::CardColorId(1), &ctx);
    assert!(missing.is_err());

    let _ = ctx.cards.insert(Card::new("C1", CardColor::Blue));
    let exists = v(&Arg::CardColorId(1), &ctx);
    assert!(exists.is_ok());
}

#[test]
fn validate_event_recurring_days_respects_recurring_flag() {
    let pat = EventPat::Base;
    let args_non_recurring = vec![
        Arg::Bool(Bool(false)),
        Arg::Name("One".into()),
        Arg::AtSymbol,
        Arg::DaysOfWeek(vec![DayOfWeek::Mon, DayOfWeek::Tue]),
        Arg::TimeRange(sample_time_range()),
    ];
    assert!(validate_event_recurring_days(&args_non_recurring, pat).is_err());

    let args_recurring = vec![
        Arg::Bool(Bool(true)),
        Arg::Name("Two".into()),
        Arg::AtSymbol,
        Arg::DaysOfWeek(vec![DayOfWeek::Mon, DayOfWeek::Tue]),
        Arg::TimeRange(sample_time_range()),
    ];
    assert!(validate_event_recurring_days(&args_recurring, pat).is_ok());
}

#[test]
fn default_days_for_varies_by_recurring() {
    assert_eq!(default_days_for(true).len(), DayOfWeek::iter().count());
    assert_eq!(default_days_for(false).len(), 1);
}

// ---------- card ----------
#[test]
fn card_builder_creates_and_modifies() {
    let args = vec![Arg::Name("Card".into()), Arg::CardColor(CardColor::Red)];
    let builder = CardBuilder;
    let card = builder
        .create(&args, CardPat::Base)
        .expect("card should build");
    assert_eq!(card.name, "Card");
    assert_eq!(card.color, CardColor::Red);

    let mut ctx = ctx();
    let stored_id = { ctx.cards.insert(card).id };
    let args_mod = vec![
        Arg::EntityType(EntityType::Card),
        Arg::Int(stored_id),
        Arg::Name("Updated".into()),
        Arg::CardColor(CardColor::Blue),
    ];
    let updated = CardSpec::new()
        .modify(&mut ctx, &args_mod, stored_id)
        .expect("modify should succeed");
    assert_eq!(updated.name, "Updated");
    assert_eq!(updated.color, CardColor::Blue);
}

// ---------- task ----------
#[test]
fn task_builder_creates_and_modifies() {
    let date = future_date();
    let args = vec![
        Arg::Name("Task".into()),
        Arg::Int(3),
        Arg::AtSymbol,
        Arg::Date(date.clone()),
    ];
    let task = TaskBuilder
        .create(&args, TaskPat::Base)
        .expect("task should build");
    assert_eq!(task.name, "Task");
    assert_eq!(task.hours, 3.0);
    assert_eq!(task.date, date);

    let mut ctx = ctx();
    let stored_id = { ctx.tasks.insert(task).id };
    let args_mod = vec![
        Arg::EntityType(EntityType::Task),
        Arg::Int(stored_id),
        Arg::Name("New".into()),
        Arg::Int(5),
        Arg::AtSymbol,
        Arg::Date(future_date()),
    ];
    let updated = TaskSpec::new()
        .modify(&mut ctx, &args_mod, stored_id)
        .expect("modify should succeed");
    assert_eq!(updated.name, "New");
    assert_eq!(updated.hours, 5.0);
    assert_eq!(updated.remaining_hours, 5.0);
    assert!(updated.subtasks.is_empty());
}

// ---------- event ----------
#[test]
fn event_builder_creates_event() {
    let args = vec![
        Arg::Bool(Bool(true)),
        Arg::Name("Meet".into()),
        Arg::AtSymbol,
        Arg::DaysOfWeek(vec![DayOfWeek::Mon, DayOfWeek::Wed]),
        Arg::TimeRange(sample_time_range()),
        Arg::Date(future_date()),
    ];
    let event = EventBuilder
        .create(&args, EventPat::Base)
        .expect("event should build");
    assert!(event.recurring);
    assert_eq!(event.name, "Meet");
    assert_eq!(event.days, vec![DayOfWeek::Mon, DayOfWeek::Wed]);
}

#[test]
fn event_builder_modifies_event() {
    let base_args = vec![
        Arg::Bool(Bool(false)),
        Arg::Name("Initial".into()),
        Arg::AtSymbol,
        Arg::DaysOfWeek(vec![DayOfWeek::Mon]),
        Arg::TimeRange(sample_time_range()),
        Arg::Date(future_date()),
    ];
    let event = EventBuilder
        .create(&base_args, EventPat::Base)
        .expect("event should build");

    let mut ctx = ctx();
    let stored_id = { ctx.events.insert(event).id };

    let modify_args = vec![
        Arg::EntityType(EntityType::Event),
        Arg::Int(stored_id),
        Arg::Bool(Bool(true)),
        Arg::Name("Updated".into()),
        Arg::AtSymbol,
        Arg::DaysOfWeek(vec![DayOfWeek::Tue, DayOfWeek::Thu]),
        Arg::TimeRange(TimeRange::try_from_str("9AM-10AM").unwrap()),
    ];

    let updated = EventSpec::new()
        .modify(&mut ctx, &modify_args, stored_id)
        .expect("modify should succeed");

    assert_eq!(updated.name, "Updated");
    assert!(updated.recurring);
    assert_eq!(updated.days, vec![DayOfWeek::Tue, DayOfWeek::Thu]);
    assert_eq!(
        updated.time_range,
        TimeRange::try_from_str("9AM-10AM").unwrap()
    );
}

#[test]
fn entity_specs_match_patterns() {
    let task_args = vec![
        Arg::Name("T".into()),
        Arg::Int(1),
        Arg::AtSymbol,
        Arg::Date(future_date()),
    ];
    // ensure EntitySpec machinery chooses the expected pattern
    assert_eq!(
        TaskSpec::new()
            .arg_schema()
            .patterns_for(EntityActionType::Add),
        vec![TaskPat::Base]
    );

    let card_args = vec![Arg::Name("C".into()), Arg::CardColor(CardColor::Black)];
    assert_eq!(
        CardSpec::new()
            .arg_schema()
            .patterns_for(EntityActionType::Add),
        vec![CardPat::Base]
    );

    let event_args = vec![
        Arg::Bool(Bool(false)),
        Arg::Name("E".into()),
        Arg::AtSymbol,
        Arg::TimeRange(sample_time_range()),
    ];
    assert_eq!(
        EventSpec::new()
            .arg_schema()
            .patterns_for(EntityActionType::Add),
        vec![EventPat::Base]
    );

    // smoke test: TaskSpec create should succeed with matching pattern
    let ctx = ctx();
    let task = TaskSpec::new().create(&ctx, &task_args).unwrap();
    assert_eq!(task.name, "T");

    let card = CardSpec::new().create(&ctx, &card_args).unwrap();
    assert_eq!(card.name, "C");

    let event = EventSpec::new().create(&ctx, &event_args).unwrap();
    assert_eq!(event.name, "E");
}
