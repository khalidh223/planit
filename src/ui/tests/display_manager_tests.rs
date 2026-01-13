use crate::config::Config;
use crate::core::{
    models::{Card, Event, Task},
    repository::Repository,
    types::{CardColor, Date, EntityType},
};
use crate::ui::{display_data::DisplayDataBuilder, display_manager::DisplayManager};
use std::fs;
use std::path::PathBuf;

fn temp_config_path() -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("planit-ui-{nanos}.json"))
}

fn write_sample_config(path: &PathBuf) {
    let json = r#"
    {
      "range": { "value": "8:00AM-6:00PM", "description": "Daily hours" },
      "task_overflow_policy": { "value": "allow", "description": "overflow" },
      "task_scheduling_order": { "value": "longest-task-first", "description": "order" },
      "schedule_start_date": { "value": null, "description": "start date" },
      "file_logging_enabled": { "value": "True", "description": "Enable writing log messages to file." }
    }
    "#;
    fs::write(path, json).unwrap();
}

#[test]
fn display_manager_config_table_matches_expected() {
    let path = temp_config_path();
    write_sample_config(&path);
    let config = Config::load_from(&path).unwrap();
    let dm = DisplayManager::new();
    let headers = ["ID", "KEY", "DESCRIPTION", "VALUE"];
    let rows: Vec<Vec<String>> = config
        .rows()
        .iter()
        .enumerate()
        .map(|(i, (k, d, v))| vec![i.to_string(), k.clone(), d.clone(), v.clone()])
        .collect();

    let mut buf = Vec::new();
    dm.printer
        .render_table(
            "Config",
            &headers,
            &rows,
            Some("No config items found."),
            None,
            &mut buf,
        )
        .unwrap();
    let output = String::from_utf8(buf).unwrap();
    let expected = fs::read_to_string("src/ui/tests/fixtures/config_table.txt").unwrap();
    assert_eq!(output, expected);
}

#[test]
fn display_manager_tasks_table_matches_expected() {
    let mut tasks = Repository::new();
    let mut cards = Repository::new();
    let card = Card::new("c", CardColor::Red);
    cards.insert(card);
    let task = Task::new(
        "task",
        1.0,
        Some(1),
        Date::try_from_str("2099-01-01").unwrap(),
    );
    tasks.insert(task);

    let builder = DisplayDataBuilder::new();
    let headers = ["ID", "NAME", "TAG", "HOURS", "DUE"];
    let rows = builder.task_rows(&tasks, &cards);

    let mut buf = Vec::new();
    DisplayManager::new()
        .printer
        .render_table(
            "Tasks",
            &headers,
            &rows,
            Some("No tasks available."),
            None,
            &mut buf,
        )
        .unwrap();
    let output = String::from_utf8(buf).unwrap();
    let expected = fs::read_to_string("src/ui/tests/fixtures/tasks_table.txt").unwrap();
    assert_eq!(output, expected);
}

#[test]
fn display_manager_events_table_matches_expected() {
    let mut events = Repository::new();
    let mut cards = Repository::new();
    let card = Card::new("c", CardColor::Blue);
    cards.insert(card);
    let event = Event::new(
        true,
        "meeting",
        Some(1),
        vec![crate::core::types::DayOfWeek::Mon],
        crate::core::types::TimeRange::try_from_str("9AM-10AM").unwrap(),
    );
    events.insert(event);

    let builder = DisplayDataBuilder::new();
    let headers = ["ID", "NAME", "TAG", "TIME", "DAYS", "RECURRING"];
    let rows = builder.event_rows(&events, &cards);

    let mut buf = Vec::new();
    DisplayManager::new()
        .printer
        .render_table(
            "Events",
            &headers,
            &rows,
            Some("No events available."),
            None,
            &mut buf,
        )
        .unwrap();
    let output = String::from_utf8(buf).unwrap();
    let expected = fs::read_to_string("src/ui/tests/fixtures/events_table.txt").unwrap();
    assert_eq!(output, expected);
}

#[test]
fn display_manager_cards_table_matches_expected() {
    let mut cards = Repository::new();
    let card = Card::new("c", CardColor::Green);
    cards.insert(card);

    let builder = DisplayDataBuilder::new();
    let headers = ["ID", "NAME", "COLOR"];
    let rows = builder.card_rows(&cards);

    let mut buf = Vec::new();
    DisplayManager::new()
        .printer
        .render_table(
            "Cards",
            &headers,
            &rows,
            Some("No cards available."),
            None,
            &mut buf,
        )
        .unwrap();
    let output = String::from_utf8(buf).unwrap();
    let expected = fs::read_to_string("src/ui/tests/fixtures/cards_table.txt").unwrap();
    assert_eq!(output, expected);
}

#[test]
fn display_manager_schedule_matches_expected() {
    let mut tasks = Repository::new();
    let mut task = Task::new(
        "task",
        1.0,
        Some(1),
        Date::try_from_str("2099-01-01").unwrap(),
    );
    task.push_subtask_with_hours(
        crate::core::types::TimeRange::try_from_str("8AM-9AM").unwrap(),
        Date::try_from_str("2099-01-01").unwrap().0,
        1.0,
    );
    tasks.insert(task);

    let mut events = Repository::new();
    let event = Event::new(
        false,
        "event",
        Some(1),
        vec![crate::core::types::DayOfWeek::Thu],
        crate::core::types::TimeRange::try_from_str("9AM-10AM").unwrap(),
    );
    events.insert(event);

    let mut cards: Repository<Card> = Repository::new();
    let card = Card::new("card", CardColor::Red);
    cards.insert(card);
    let dm = DisplayManager::new();
    let dates = vec![
        Date::try_from_str("2099-01-01").unwrap().0,
        Date::try_from_str("2099-01-02").unwrap().0,
    ];
    let mut out = Vec::new();
    dm.render_schedule_for_days(&dates, &tasks, &events, &cards, &mut out)
        .unwrap();

    let output = String::from_utf8(out).unwrap();
    let expected = fs::read_to_string("src/ui/tests/fixtures/schedule_table.txt").unwrap();
    assert_eq!(output, expected);
}

#[test]
fn display_config_centered_returns_expected_width() {
    let path = temp_config_path();
    write_sample_config(&path);
    let config = Config::load_from(&path).unwrap();
    let dm = DisplayManager::new();

    let headers = ["ID", "KEY", "DESCRIPTION", "VALUE"];
    let rows: Vec<Vec<String>> = config
        .rows()
        .iter()
        .enumerate()
        .map(|(i, (k, d, v))| vec![i.to_string(), k.clone(), d.clone(), v.clone()])
        .collect();

    let expected_table = dm.printer.compute_table_width(&headers, &rows);
    let expected = expected_table.max(dm.util.visible_width("CONFIG"));
    let width = dm.display_config_centered(&config);
    assert_eq!(width, expected);
}

#[test]
fn display_config_does_not_mutate_rows() {
    let path = temp_config_path();
    write_sample_config(&path);
    let config = Config::load_from(&path).unwrap();
    let dm = DisplayManager::new();

    let before = config.rows().len();
    dm.display_config(&config);
    let after = config.rows().len();
    assert_eq!(before, after);
}

#[test]
fn display_entities_for_tasks_does_not_mutate_repo() {
    let dm = DisplayManager::new();
    let tasks: Repository<Task> = Repository::new();
    let events: Repository<Event> = Repository::new();
    let cards: Repository<Card> = Repository::new();

    let before = tasks.len();
    dm.display_entities_for(EntityType::Task, &tasks, &events, &cards);
    let after = tasks.len();
    assert_eq!(before, after);
}

#[test]
fn display_entities_for_events_does_not_mutate_repo() {
    let dm = DisplayManager::new();
    let tasks: Repository<Task> = Repository::new();
    let events: Repository<Event> = Repository::new();
    let cards: Repository<Card> = Repository::new();

    let before = events.len();
    dm.display_entities_for(EntityType::Event, &tasks, &events, &cards);
    let after = events.len();
    assert_eq!(before, after);
}

#[test]
fn display_entities_for_cards_does_not_mutate_repo() {
    let dm = DisplayManager::new();
    let tasks: Repository<Task> = Repository::new();
    let events: Repository<Event> = Repository::new();
    let cards: Repository<Card> = Repository::new();

    let before = cards.len();
    dm.display_entities_for(EntityType::Card, &tasks, &events, &cards);
    let after = cards.len();
    assert_eq!(before, after);
}

#[test]
fn display_schedule_for_days_does_not_mutate_tasks() {
    let dm = DisplayManager::new();
    let tasks: Repository<Task> = Repository::new();
    let events: Repository<Event> = Repository::new();
    let cards: Repository<Card> = Repository::new();
    let dates = vec![Date::try_from_str("2099-01-01").unwrap().0];

    let before = tasks.len();
    dm.display_schedule_for_days(&dates, &tasks, &events, &cards);
    let after = tasks.len();
    assert_eq!(before, after);
}
