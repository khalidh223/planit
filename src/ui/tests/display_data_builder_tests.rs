use crate::core::{
    models::{BaseEntity, Card, Event, Task},
    repository::Repository,
    types::{CardColor, Date},
};
use crate::ui::display_data::DisplayDataBuilder;

#[test]
fn display_data_builder_builds_painted_rows() {
    let builder = DisplayDataBuilder::new();
    let red = CardColor::Red;
    let mut cards = Repository::new();
    let mut card = Card::new("c", red);
    card.set_id(1);
    cards.insert(card);

    let mut tasks = Repository::new();
    let mut task = Task::new("t", 2.0, Some(1), Date::try_from_str("2099-01-01").unwrap());
    task.set_id(1);
    tasks.insert(task);

    let rows = builder.task_rows(&tasks, &cards);
    assert_eq!(
        rows[0],
        vec![
            format!("{}1{}", red.ansi_fg(), CardColor::RESET),
            format!("{}t{}", red.ansi_fg(), CardColor::RESET),
            format!("{}c{}", red.ansi_fg(), CardColor::RESET),
            format!("{}2.00{}", red.ansi_fg(), CardColor::RESET),
            format!("{}2099-01-01{}", red.ansi_fg(), CardColor::RESET)
        ]
    );
}

#[test]
fn display_data_builder_sorts_schedule_rows_by_time() {
    let builder = DisplayDataBuilder::new();
    let mut tasks = Repository::new();
    let mut task = Task::new("t", 2.0, None, Date::try_from_str("2099-01-01").unwrap());
    task.push_subtask_with_hours(
        crate::core::types::TimeRange::try_from_str("9AM-10AM").unwrap(),
        Date::try_from_str("2099-01-01").unwrap().0,
        1.0,
    );
    task.push_subtask_with_hours(
        crate::core::types::TimeRange::try_from_str("8AM-9AM").unwrap(),
        Date::try_from_str("2099-01-01").unwrap().0,
        1.0,
    );
    tasks.insert(task);

    let sections = builder.build_schedule_sections(
        &[Date::try_from_str("2099-01-01").unwrap().0],
        &tasks,
        &Repository::<Event>::new(),
        &Repository::<Card>::new(),
    );
    let rows = &sections[0].rows;
    assert_eq!(rows[0][4], "8:00AM-9:00AM");
    assert_eq!(rows[1][4], "9:00AM-10:00AM");
}
