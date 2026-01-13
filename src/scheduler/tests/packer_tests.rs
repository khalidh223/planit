use crate::core::models::{BaseEntity, FreeTimeBlock, Task};
use crate::core::types::{Date, TimeRange};
use crate::logging::Logger;
use crate::scheduler::LoggerObserver;
use crate::scheduler::packer::{BlockPacker, FirstFitPacker, PackOutcome};
use chrono::NaiveDateTime;

#[test]
fn packer_returns_partial_when_free_time_insufficient() {
    let mut packer = FirstFitPacker;
    let mut task = Task::new("t", 3.0, None, Date(super::sample_date()));
    task.set_id(1);
    let date = task.date.0;
    let block = TimeRange::try_from_str("8AM-10AM").unwrap();
    let mut free = vec![FreeTimeBlock::new(
        NaiveDateTime::new(date, block.start),
        NaiveDateTime::new(date, block.end),
    )];

    let logger = Logger::new();
    let obs = LoggerObserver {
        logger: logger.clone(),
    };
    let outcome = packer.pack(&mut task, date, &mut free, &obs);

    assert!(matches!(outcome, PackOutcome::Partial));
    assert_eq!(task.subtasks.len(), 1);
    assert!(task.remaining_hours > 0.0);
}

#[test]
fn packer_returns_full_when_additional_block_available() {
    let mut packer = FirstFitPacker;
    let mut task = Task::new("t", 2.0, None, Date(super::sample_date()));
    task.set_id(2);
    let date = task.date.0;
    let mut free = vec![
        FreeTimeBlock::new(
            NaiveDateTime::new(date, TimeRange::try_from_str("8AM-9AM").unwrap().start),
            NaiveDateTime::new(date, TimeRange::try_from_str("8AM-9AM").unwrap().end),
        ),
        FreeTimeBlock::new(
            NaiveDateTime::new(date, TimeRange::try_from_str("9AM-10AM").unwrap().start),
            NaiveDateTime::new(date, TimeRange::try_from_str("9AM-10AM").unwrap().end),
        ),
    ];

    let logger = Logger::new();
    let obs = LoggerObserver {
        logger: logger.clone(),
    };
    let outcome = packer.pack(&mut task, date, &mut free, &obs);

    assert!(matches!(outcome, PackOutcome::Full));
    assert_eq!(task.remaining_hours, 0.0);
}
