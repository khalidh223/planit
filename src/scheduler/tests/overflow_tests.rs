use crate::core::models::{BaseEntity, Task};
use crate::core::types::{Date, TaskOverflowPolicy, TimeRange};
use crate::scheduler::overflow::{
    AllowOverflow, BlockOverflow, OverflowPolicyHandler, make_overflow_handler,
};

#[test]
fn allow_overflow_marks_last_subtask() {
    let mut task = Task::new("a", 2.0, None, Date(super::sample_date()));
    task.set_id(1);
    task.push_subtask_with_hours(
        TimeRange::try_from_str("8AM-9AM").unwrap(),
        task.date.0,
        1.0,
    );
    let allow = AllowOverflow;
    allow.handle(&mut task, true).unwrap();
    assert!(task.subtasks.last().unwrap().overflow);
}

#[test]
fn block_overflow_allows_when_no_remaining() {
    let mut task = Task::new("b", 1.0, None, Date(super::sample_date()));
    task.remaining_hours = 0.0;
    let block = BlockOverflow;
    assert!(block.handle(&mut task, true).is_ok());
}

#[test]
fn block_overflow_errors_when_remaining() {
    let mut task = Task::new("b", 1.0, None, Date(super::sample_date()));
    task.remaining_hours = 0.5;
    let block = BlockOverflow;
    assert!(block.handle(&mut task, true).is_err());
}

#[test]
fn factory_returns_expected_handler() {
    let mut task = Task::new("a", 1.0, None, Date(super::sample_date()));
    let handler = make_overflow_handler(TaskOverflowPolicy::Allow);
    assert!(handler.handle(&mut task, false).is_ok());
}
