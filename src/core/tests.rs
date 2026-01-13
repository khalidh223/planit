use super::{
    context::AppContext,
    models::{BaseEntity, Card, Event, FreeTimeBlock, Task},
    persist::{SaveFile, load_state, save_state},
    repository::{Repository, Sort},
    types::{
        Bool, CardColor, Date, DayOfWeek, EntityActionType, EntityType, GlobalCommand,
        TaskOverflowPolicy, TaskSchedulingOrder, TimeRange,
    },
};
use crate::core::cli::CliPaths;
use crate::errors::Error;
use chrono::{Datelike, NaiveDate, Timelike};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_save_path(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("planit-save-{name}-{nanos}.json"))
}

fn write_save_file(path: &PathBuf, save_file: &SaveFile) {
    let contents = serde_json::to_string_pretty(save_file).unwrap();
    fs::write(path, contents).unwrap();
}

// ---------- types.rs ----------
#[test]
fn parses_entity_types_and_actions() {
    assert_eq!(EntityType::try_from("task").unwrap(), EntityType::Task);
    assert!(EntityType::try_from("bogus").is_err());

    assert_eq!(
        EntityActionType::try_from("mod").unwrap(),
        EntityActionType::Modify
    );
    assert!(EntityActionType::try_from("noop").is_err());

    assert_eq!(GlobalCommand::try_from("man").unwrap(), GlobalCommand::Man);
    assert!(GlobalCommand::try_from("nope").is_err());
}

#[test]
fn parses_dates_and_timeranges() {
    let d = Date::try_from_str("2025-01-01").unwrap();
    assert_eq!(d.to_string(), "2025-01-01");

    let tr = TimeRange::try_from_str("8AM-10AM").unwrap();
    assert!(tr.start < tr.end);
    assert!(TimeRange::try_from_str("bad").is_err());
}

#[test]
fn date_accepts_multiple_formats_and_defaults_year() {
    // mm-dd
    let d1 = Date::try_from_str("12-31").unwrap();
    assert_eq!(d1.0.month(), 12);
    assert_eq!(d1.0.day(), 31);

    // slash format
    let d2 = Date::try_from_str("01/02/2025").unwrap();
    assert_eq!(d2.to_string(), "2025-01-02");

    // invalid still errors
    assert!(Date::try_from_str("13/40").is_err());
}

#[test]
fn timerange_parses_tokens_and_validates_order() {
    let tr = TimeRange::try_from_str("8:30AM-1PM").unwrap();
    assert_eq!(tr.start.hour(), 8);
    assert_eq!(tr.start.minute(), 30);
    assert_eq!(tr.end.hour(), 13);
    assert_eq!(tr.end.minute(), 0);

    // ensure defaults to AM/PM when missing meridian on end
    let tr2 = TimeRange::try_from_str("8AM-9").unwrap();
    // Without explicit meridian, parser infers PM for end token.
    assert_eq!(tr2.end.hour(), 21);

    // invalid order
    assert!(TimeRange::try_from_str("5PM-4PM").is_err());
    // missing dash
    assert!(TimeRange::try_from_str("8AM").is_err());
}

#[test]
fn parses_bool_and_policy_enums() {
    assert_eq!(Bool::try_from_str("true").unwrap(), Bool(true));
    assert!(Bool::try_from_str("not-bool").is_err());

    assert_eq!(
        TaskOverflowPolicy::try_from("block").unwrap(),
        TaskOverflowPolicy::Block
    );
    assert!(TaskOverflowPolicy::try_from("x").is_err());
    assert!(TaskOverflowPolicy::Block.help().contains("Fail"));

    assert_eq!(
        TaskSchedulingOrder::try_from("longest-task-first").unwrap(),
        TaskSchedulingOrder::LongestTaskFirst
    );
    assert!(TaskSchedulingOrder::DueOnly.help().contains("due"));
}

#[test]
fn parses_day_of_week() {
    assert_eq!(DayOfWeek::try_from("mon").unwrap(), DayOfWeek::Mon);
    assert!(DayOfWeek::try_from("zzz").is_err());
}

// ---------- models.rs ----------
#[test]
fn task_modify_resets_state_and_subtasks() {
    let mut task = Task::new("t", 4.0, None, Date::try_from_str("2025-01-01").unwrap());
    task.push_subtask_with_hours(
        TimeRange::try_from_str("8AM-9AM").unwrap(),
        NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
        1.0,
    );
    assert!(task.remaining_hours < 4.0);

    task.modify(
        "new",
        2.0,
        Some(1),
        Date::try_from_str("2025-02-02").unwrap(),
    );
    assert_eq!(task.name, "new");
    assert_eq!(task.hours, 2.0);
    assert_eq!(task.remaining_hours, 2.0);
    assert!(task.subtasks.is_empty());
}

#[test]
fn subtask_hours_and_scheduled_time_display() {
    let date = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
    let tr = TimeRange::try_from_str("8AM-10AM").unwrap();
    let st = crate::core::types::ScheduledTime {
        date,
        time_range: tr.clone(),
    };
    assert!((st.duration_in_hours() - 2.0).abs() < f32::EPSILON);
    assert!(st.to_string().contains("2025-01-01"));

    let mut task = Task::new("t", 3.0, None, Date(date));
    task.push_subtask_with_hours(tr.clone(), date, 1.5);
    assert_eq!(task.subtasks.len(), 1);
    assert!((task.subtasks[0].hours() - 2.0).abs() < 1e-6);
}

#[test]
fn event_active_on_date_respects_weekday() {
    let event = Event::new(
        true,
        "e",
        None,
        vec![DayOfWeek::Mon, DayOfWeek::Wed],
        TimeRange::try_from_str("9AM-10AM").unwrap(),
    );
    let monday = NaiveDate::from_ymd_opt(2025, 1, 6).unwrap(); // Monday
    let friday = NaiveDate::from_ymd_opt(2025, 1, 10).unwrap(); // Friday
    assert!(event.is_active_on_date(monday));
    assert!(!event.is_active_on_date(friday));
}

#[test]
fn free_time_block_computes_remaining() {
    let start = NaiveDate::from_ymd_opt(2025, 1, 1)
        .unwrap()
        .and_hms_opt(8, 0, 0)
        .unwrap();
    let end = start + chrono::Duration::hours(2);
    let fb = FreeTimeBlock::new(start, end);
    assert!((fb.remaining_free_time - 2.0).abs() < f32::EPSILON);
}

// ---------- repository.rs ----------
#[test]
fn repository_inserts_and_gets_entities() {
    let mut repo = Repository::<Task>::new();
    let t1 = repo.insert(Task::new(
        "a",
        1.0,
        None,
        Date::try_from_str("2025-01-01").unwrap(),
    ));
    // Drop first borrow before second insert.
    let t1_id = t1.id;
    let t2_id = {
        let t = Task::new("b", 2.0, None, Date::try_from_str("2025-01-02").unwrap());
        repo.insert(t).id
    };
    assert_eq!(t1_id, 1);
    assert_eq!(t2_id, 2);

    assert_eq!(repo.get(t1_id).unwrap().name, "a");
    assert!(repo.get(99).is_err());
}

#[test]
fn repository_values_sort_and_query_filter() {
    let mut repo = Repository::<Task>::new();
    repo.insert(Task::new(
        "a",
        1.0,
        None,
        Date::try_from_str("2025-01-01").unwrap(),
    ));
    repo.insert(Task::new(
        "b",
        2.0,
        None,
        Date::try_from_str("2025-01-02").unwrap(),
    ));

    let asc = repo.values(Sort::IdAsc);
    assert_eq!(asc[0].id, 1);
    let desc = repo.values(Sort::IdDesc);
    assert_eq!(desc[0].id, 2);

    let filtered = repo.query().r#where(|t| t.id == 2).collect();
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].name, "b");
}

#[test]
fn repository_delete_clear_and_exists() {
    let mut repo = Repository::<Task>::new();
    repo.insert(Task::new(
        "a",
        1.0,
        None,
        Date::try_from_str("2025-01-01").unwrap(),
    ));
    assert!(repo.query().exists());
    let deleted = repo.delete(1).unwrap();
    assert_eq!(deleted.name, "a");
    assert!(!repo.query().exists());

    repo.insert(Task::new(
        "b",
        1.0,
        None,
        Date::try_from_str("2025-01-01").unwrap(),
    ));
    repo.clear();
    assert_eq!(repo.len(), 0);
}

#[test]
fn repository_query_mut_updates_in_place() {
    let mut repo = Repository::<Task>::new();
    repo.insert(Task::new(
        "a",
        1.0,
        None,
        Date::try_from_str("2025-01-01").unwrap(),
    ));

    repo.query_mut().for_each_mut(|t| {
        t.hours += 1.0;
        t.remaining_hours = t.hours;
    });

    assert_eq!(repo.get(1).unwrap().hours, 2.0);
}

#[test]
fn repository_ids_ordering() {
    let mut repo = Repository::<Task>::new();
    repo.insert(Task::new(
        "a",
        1.0,
        None,
        Date::try_from_str("2025-01-01").unwrap(),
    ));
    repo.insert(Task::new(
        "b",
        1.0,
        None,
        Date::try_from_str("2025-01-01").unwrap(),
    ));

    let mut ids = repo.query().ids();
    ids.sort();
    assert_eq!(ids, vec![1, 2]);
}

#[test]
fn repository_insert_with_id_sets_next_id() {
    let mut repo = Repository::<Task>::new();
    let mut task = Task::new("a", 1.0, None, Date::try_from_str("2025-01-01").unwrap());
    task.set_id(5);

    repo.insert_with_id(task).unwrap();
    assert_eq!(repo.peek_next_id(), 6);
    assert_eq!(repo.get(5).unwrap().id, 5);
}

#[test]
fn repository_insert_with_id_rejects_non_positive() {
    let mut repo = Repository::<Task>::new();
    let mut task = Task::new("a", 1.0, None, Date::try_from_str("2025-01-01").unwrap());
    task.set_id(0);

    let err = repo.insert_with_id(task).unwrap_err();
    match err {
        Error::Parse(msg) => assert_eq!(msg, "ID must be positive."),
        other => panic!("expected parse error, got {other:?}"),
    }
}

#[test]
fn repository_insert_with_id_rejects_duplicate() {
    let mut repo = Repository::<Task>::new();
    let mut task = Task::new("a", 1.0, None, Date::try_from_str("2025-01-01").unwrap());
    task.set_id(2);
    repo.insert_with_id(task).unwrap();

    let mut dup = Task::new("b", 1.0, None, Date::try_from_str("2025-01-02").unwrap());
    dup.set_id(2);
    let err = repo.insert_with_id(dup).unwrap_err();
    match err {
        Error::Parse(msg) => assert_eq!(msg, "Entity with id 2 already exists."),
        other => panic!("expected parse error, got {other:?}"),
    }
}

#[test]
fn repository_begin_stage_rejects_nested_transactions() {
    let mut repo = Repository::<Task>::new();
    repo.begin_stage(false).unwrap();
    let err = repo.begin_stage(false).unwrap_err();
    match err {
        Error::Parse(msg) => assert_eq!(msg, "Transaction already in progress."),
        other => panic!("expected parse error, got {other:?}"),
    }
}

#[test]
fn repository_stage_insert_is_deferred() {
    let mut repo = Repository::<Task>::new();
    repo.begin_stage(false).unwrap();
    repo.insert(Task::new(
        "a",
        1.0,
        None,
        Date::try_from_str("2025-01-01").unwrap(),
    ));

    assert_eq!(repo.len(), 0);
    assert_eq!(repo.staged_pending().unwrap().len(), 1);
}

#[test]
fn repository_discard_stage_restores_next_id() {
    let mut repo = Repository::<Task>::new();
    repo.begin_stage(false).unwrap();
    repo.insert(Task::new(
        "a",
        1.0,
        None,
        Date::try_from_str("2025-01-01").unwrap(),
    ));
    assert_eq!(repo.peek_next_id(), 2);

    repo.discard_stage();
    assert_eq!(repo.peek_next_id(), 1);
    assert_eq!(repo.len(), 0);
}

#[test]
fn repository_staged_effective_ids_returns_union_without_clear() {
    let mut repo = Repository::<Task>::new();
    repo.insert(Task::new(
        "a",
        1.0,
        None,
        Date::try_from_str("2025-01-01").unwrap(),
    ));
    repo.begin_stage(false).unwrap();
    repo.insert(Task::new(
        "b",
        1.0,
        None,
        Date::try_from_str("2025-01-02").unwrap(),
    ));

    let ids = repo.staged_effective_ids().unwrap();
    let expected: std::collections::HashSet<i32> = [1, 2].into_iter().collect();
    assert_eq!(ids, expected);
}

#[test]
fn repository_staged_effective_ids_respects_clear_existing() {
    let mut repo = Repository::<Task>::new();
    repo.insert(Task::new(
        "a",
        1.0,
        None,
        Date::try_from_str("2025-01-01").unwrap(),
    ));
    repo.begin_stage(true).unwrap();

    let ids = repo.staged_effective_ids().unwrap();
    assert!(ids.is_empty());
}

#[test]
fn repository_exists_including_staged_ignores_existing_when_cleared() {
    let mut repo = Repository::<Task>::new();
    repo.insert(Task::new(
        "a",
        1.0,
        None,
        Date::try_from_str("2025-01-01").unwrap(),
    ));
    repo.begin_stage(true).unwrap();

    assert!(!repo.exists_including_staged(1));
}

#[test]
fn repository_prepare_commit_applies_pending() {
    let mut repo = Repository::<Task>::new();
    repo.begin_stage(false).unwrap();
    repo.insert(Task::new(
        "a",
        1.0,
        None,
        Date::try_from_str("2025-01-01").unwrap(),
    ));

    let prepared = repo.prepare_commit().unwrap();
    repo.apply_prepared(prepared);
    assert_eq!(repo.len(), 1);
    assert_eq!(repo.get(1).unwrap().name, "a");
}

#[test]
fn repository_prepare_commit_errors_without_stage() {
    let repo = Repository::<Task>::new();
    let err = repo.prepare_commit().unwrap_err();
    match err {
        Error::Parse(msg) => assert_eq!(msg, "No active transaction to commit."),
        other => panic!("expected parse error, got {other:?}"),
    }
}

#[test]
fn repository_staged_effective_ids_errors_without_stage() {
    let repo = Repository::<Task>::new();
    let err = repo.staged_effective_ids().unwrap_err();
    match err {
        Error::Parse(msg) => assert_eq!(msg, "No active transaction to inspect."),
        other => panic!("expected parse error, got {other:?}"),
    }
}

#[test]
fn repository_query_exists_respects_filter() {
    let mut repo = Repository::<Task>::new();
    repo.insert(Task::new(
        "a",
        1.0,
        None,
        Date::try_from_str("2025-01-01").unwrap(),
    ));

    assert!(!repo.query().r#where(|t| t.name == "missing").exists());
}

#[test]
fn repository_query_order_with_uses_custom_comparator() {
    let mut repo = Repository::<Task>::new();
    repo.insert(Task::new(
        "b",
        1.0,
        None,
        Date::try_from_str("2025-01-01").unwrap(),
    ));
    repo.insert(Task::new(
        "a",
        1.0,
        None,
        Date::try_from_str("2025-01-02").unwrap(),
    ));

    let ordered = repo
        .query()
        .order_with(|a, b| a.name.cmp(&b.name))
        .collect();
    assert_eq!(ordered[0].name, "a");
}

#[test]
fn repository_restore_next_id_sets_value() {
    let mut repo = Repository::<Task>::new();
    repo.restore_next_id(10);
    assert_eq!(repo.peek_next_id(), 10);
}

#[test]
fn repository_values_mut_updates_items() {
    let mut repo = Repository::<Task>::new();
    repo.insert(Task::new(
        "a",
        1.0,
        None,
        Date::try_from_str("2025-01-01").unwrap(),
    ));

    for t in repo.values_mut() {
        t.name = "updated".to_string();
    }

    assert_eq!(repo.get(1).unwrap().name, "updated");
}

#[test]
fn repository_get_mut_errors_when_missing() {
    let mut repo = Repository::<Task>::new();
    let err = repo.get_mut(99).unwrap_err();
    match err {
        Error::Parse(msg) => assert_eq!(msg, "Entity with id 99 not found."),
        other => panic!("expected parse error, got {other:?}"),
    }
}

#[test]
fn repository_staged_pending_is_none_without_stage() {
    let repo = Repository::<Task>::new();
    assert!(repo.staged_pending().is_none());
}

#[test]
fn repository_begin_stage_with_clear_resets_next_id() {
    let mut repo = Repository::<Task>::new();
    repo.insert(Task::new(
        "a",
        1.0,
        None,
        Date::try_from_str("2025-01-01").unwrap(),
    ));

    repo.begin_stage(true).unwrap();
    assert_eq!(repo.peek_next_id(), 1);
}

#[test]
fn repository_exists_including_staged_includes_pending_when_not_cleared() {
    let mut repo = Repository::<Task>::new();
    repo.insert(Task::new(
        "a",
        1.0,
        None,
        Date::try_from_str("2025-01-01").unwrap(),
    ));
    repo.begin_stage(false).unwrap();
    repo.insert(Task::new(
        "b",
        1.0,
        None,
        Date::try_from_str("2025-01-02").unwrap(),
    ));

    assert!(repo.exists_including_staged(1));
    assert!(repo.exists_including_staged(2));
}

// ---------- context.rs ----------
#[test]
fn app_context_initializes_defaults() {
    let ctx = AppContext::new();
    assert!(!ctx.startup_displayed);
    assert!(ctx.logger.log_path().is_none());
}

// ---------- card color ----------
#[test]
fn card_color_paint_wraps_string() {
    let painted = CardColor::Red.paint("hello");
    assert!(painted.contains("hello"));
    assert!(painted.contains(CardColor::RESET));
}

// ---------- persist.rs ----------
#[test]
fn save_state_writes_expected_tokens() {
    let mut cards = Repository::<Card>::new();
    let mut tasks = Repository::<Task>::new();
    let mut events = Repository::<Event>::new();

    let card = cards.insert(Card::new("Focus", CardColor::Blue));
    let task = Task::new(
        "Deep Work",
        2.0,
        Some(card.id),
        Date::try_from_str("2099-01-02").unwrap(),
    );
    tasks.insert(task);
    let event = Event::new(
        true,
        "Sync",
        Some(card.id),
        vec![DayOfWeek::Mon, DayOfWeek::Wed],
        TimeRange::try_from_str("9:00AM-10:00AM").unwrap(),
    );
    events.insert(event);

    let path = temp_save_path("tokens");
    let saved = save_state(&tasks, &events, &cards, &path).unwrap();
    let contents = fs::read_to_string(saved).unwrap();
    let save_file: SaveFile = serde_json::from_str(&contents).unwrap();

    assert_eq!(
        save_file.cards,
        vec![vec![String::from("\"Focus\""), String::from("BLUE")]]
    );
    assert_eq!(
        save_file.tasks,
        vec![vec![
            String::from("\"Deep Work\""),
            String::from("2"),
            String::from("+C1"),
            String::from("@"),
            String::from("2099-01-02")
        ]]
    );
    assert_eq!(
        save_file.events,
        vec![vec![
            String::from("True"),
            String::from("\"Sync\""),
            String::from("+C1"),
            String::from("@"),
            String::from("MON,"),
            String::from("WED"),
            String::from("9:00AM-10:00AM")
        ]]
    );
}

#[test]
fn load_state_inserts_cards() {
    let path = temp_save_path("cards");
    let save_file = SaveFile {
        cards: vec![vec!["\"Card\"".into(), "RED".into()]],
        events: Vec::new(),
        tasks: Vec::new(),
    };
    write_save_file(&path, &save_file);

    let mut ctx = AppContext::new();
    ctx.logger.set_file_logging_enabled(false);
    load_state(&mut ctx, &path).unwrap();

    assert_eq!(ctx.cards.len(), 1);
    let card = ctx.cards.get(1).unwrap();
    assert_eq!(card.name, "Card");
}

#[test]
fn load_state_inserts_tasks_with_card_reference() {
    let path = temp_save_path("tasks");
    let save_file = SaveFile {
        cards: vec![vec!["\"Tag\"".into(), "GREEN".into()]],
        events: Vec::new(),
        tasks: vec![vec![
            "\"Task\"".into(),
            "1".into(),
            "+C1".into(),
            "@".into(),
            "2099-01-01".into(),
        ]],
    };
    write_save_file(&path, &save_file);

    let mut ctx = AppContext::new();
    ctx.logger.set_file_logging_enabled(false);
    load_state(&mut ctx, &path).unwrap();

    assert_eq!(ctx.tasks.len(), 1);
    let task = ctx.tasks.get(1).unwrap();
    assert_eq!(task.card_id, Some(1));
}

#[test]
fn load_state_inserts_events_with_card_reference() {
    let path = temp_save_path("events");
    let save_file = SaveFile {
        cards: vec![vec!["\"Tag\"".into(), "RED".into()]],
        events: vec![vec![
            "True".into(),
            "\"Event\"".into(),
            "+C1".into(),
            "@".into(),
            "MON".into(),
            "8:00AM-9:00AM".into(),
        ]],
        tasks: Vec::new(),
    };
    write_save_file(&path, &save_file);

    let mut ctx = AppContext::new();
    ctx.logger.set_file_logging_enabled(false);
    load_state(&mut ctx, &path).unwrap();

    assert_eq!(ctx.events.len(), 1);
    let event = ctx.events.get(1).unwrap();
    assert_eq!(event.card_id, Some(1));
}

#[test]
fn load_state_rolls_back_on_error() {
    let path = temp_save_path("rollback");
    let save_file = SaveFile {
        cards: Vec::new(),
        events: vec![vec![
            "True".into(),
            "\"Event\"".into(),
            "+C1".into(),
            "@".into(),
            "MON".into(),
            "8:00AM-9:00AM".into(),
        ]],
        tasks: Vec::new(),
    };
    write_save_file(&path, &save_file);

    let mut ctx = AppContext::new();
    ctx.logger.set_file_logging_enabled(false);
    let result = load_state(&mut ctx, &path);

    assert!(result.is_err());
    assert_eq!(ctx.cards.len(), 0);
    assert_eq!(ctx.events.len(), 0);
    assert_eq!(ctx.tasks.len(), 0);
}

#[test]
fn cli_paths_defaults_when_no_args() {
    let paths = CliPaths::from_args(std::iter::empty()).unwrap();
    assert_eq!(paths.config_path, PathBuf::from("config.json"));
    assert_eq!(paths.schedules_dir, PathBuf::from("schedules"));
    assert_eq!(paths.logs_dir, PathBuf::from("logs"));
}

#[test]
fn cli_paths_overrides_all_paths() {
    let args = vec![
        "--config".to_string(),
        "/tmp/cfg.json".to_string(),
        "--schedules".to_string(),
        "/tmp/schedules".to_string(),
        "--logs".to_string(),
        "/tmp/logs".to_string(),
    ];
    let paths = CliPaths::from_args(args.into_iter()).unwrap();
    assert_eq!(paths.config_path, PathBuf::from("/tmp/cfg.json"));
    assert_eq!(paths.schedules_dir, PathBuf::from("/tmp/schedules"));
    assert_eq!(paths.logs_dir, PathBuf::from("/tmp/logs"));
}

#[test]
fn cli_paths_errors_on_unknown_flag() {
    let args = vec!["--nope".to_string()];
    let err = CliPaths::from_args(args.into_iter()).unwrap_err();
    assert!(err.contains("Unknown argument"));
}

#[test]
fn cli_paths_errors_on_missing_value() {
    let args = vec!["--config".to_string()];
    let err = CliPaths::from_args(args.into_iter()).unwrap_err();
    assert_eq!(err, "Missing value for --config");
}
