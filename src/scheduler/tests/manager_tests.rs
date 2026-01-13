use crate::core::models::Task;
use crate::core::types::Date;
use crate::logging::Logger;
use crate::scheduler::{
    LoggerObserver, ScheduleManager,
    packer::{PackOutcome, ScheduleObserver},
};
use chrono::NaiveDate;

#[test]
fn logger_observer_emits_messages_for_outcomes() {
    let logger = Logger::new();
    let obs = LoggerObserver { logger };
    let date = super::sample_date();
    // ensure all branches execute without error
    obs.task_scheduled(1, date, 2.0, 2.0, &PackOutcome::Full);
    obs.task_scheduled(2, date, 3.0, 1.5, &PackOutcome::Full);
    obs.task_scheduled(3, date, 3.0, 1.0, &PackOutcome::Partial);
    obs.task_scheduled(4, date, 1.0, 0.0, &PackOutcome::None);
}

#[test]
fn schedule_manager_compute_schedule_runs() {
    let mut ctx = super::make_ctx();
    let task = Task::new(
        "sched",
        1.0,
        None,
        Date(NaiveDate::from_ymd_opt(2099, 1, 1).unwrap()),
    );
    ctx.tasks.insert(task);

    let mut mgr = ScheduleManager::new(&mut ctx);
    mgr.compute_schedule().unwrap();
    let scheduled = ctx.tasks.get(1).unwrap();
    assert!(!scheduled.subtasks.is_empty());
    assert!(scheduled.remaining_hours <= 0.0);
}
