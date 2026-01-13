use crate::core::models::{BaseEntity, Task};
use crate::core::types::{Date, TaskSchedulingOrder};
use crate::scheduler::comparator::make_task_order_comparator;

fn sample_tasks() -> (Task, Task) {
    let date1 = super::sample_date();
    let date2 = date1.succ_opt().unwrap();
    let mut t1 = Task::new("a", 2.0, None, Date(date1));
    t1.set_id(1);
    let mut t2 = Task::new("b", 1.0, None, Date(date2));
    t2.set_id(2);
    (t1, t2)
}

#[test]
fn longest_task_first_prioritizes_earlier_due_date() {
    let (t1, t2) = sample_tasks();
    let cmp = make_task_order_comparator(TaskSchedulingOrder::LongestTaskFirst);
    assert!(cmp.cmp(&t1, &t2).is_lt());
}

#[test]
fn shortest_task_first_prefers_shorter_remaining_when_dates_equal() {
    let (t1, mut t2) = sample_tasks();
    t2.date = t1.date.clone();
    let cmp = make_task_order_comparator(TaskSchedulingOrder::ShortestTaskFirst);
    assert!(cmp.cmp(&t1, &t2).is_lt());
}

#[test]
fn due_only_tiebreaks_by_id_when_dates_equal() {
    let (mut t1, mut t2) = sample_tasks();
    t2.date = t1.date.clone();
    t1.set_id(3);
    let cmp = make_task_order_comparator(TaskSchedulingOrder::DueOnly);
    assert!(cmp.cmp(&t1, &t2).is_gt());
}

#[test]
fn longest_task_first_orders_by_remaining_when_dates_equal() {
    let date = super::sample_date();
    let mut t1 = Task::new("a", 4.0, None, Date(date));
    t1.set_id(1);
    let mut t2 = Task::new("b", 2.0, None, Date(date));
    t2.set_id(2);

    let cmp = make_task_order_comparator(TaskSchedulingOrder::LongestTaskFirst);
    assert!(cmp.cmp(&t1, &t2).is_gt());
}

#[test]
fn shortest_task_first_tiebreaks_by_id_when_remaining_equal() {
    let date = super::sample_date();
    let mut t1 = Task::new("a", 2.0, None, Date(date));
    t1.set_id(2);
    let mut t2 = Task::new("b", 2.0, None, Date(date));
    t2.set_id(1);

    let cmp = make_task_order_comparator(TaskSchedulingOrder::ShortestTaskFirst);
    assert!(cmp.cmp(&t1, &t2).is_gt());
}

#[test]
fn due_only_orders_by_due_date() {
    let (t1, t2) = sample_tasks();
    let cmp = make_task_order_comparator(TaskSchedulingOrder::DueOnly);
    assert!(cmp.cmp(&t1, &t2).is_lt());
}
