use crate::core::models::{Event, FreeTimeBlock, Task};
use crate::core::types::{Date, DayOfWeek, TimeRange};
use crate::scheduler::calendar_view::CalendarView;
use chrono::NaiveDateTime;

#[test]
fn calendar_view_generates_days() {
    let start = super::sample_date();
    let days: Vec<_> = CalendarView::new()
        .with_start_date(start)
        .with_days(3)
        .days();
    assert_eq!(days.len(), 3);
    assert_eq!(days[0], start);
    assert_eq!(days[2], start + chrono::Duration::days(2));
}

#[test]
fn calendar_view_builds_free_blocks() {
    let start = super::sample_date();
    let mut ctx = super::make_ctx();

    let event = Event::new(
        true,
        "e",
        None,
        vec![
            DayOfWeek::Mon,
            DayOfWeek::Tue,
            DayOfWeek::Wed,
            DayOfWeek::Thu,
            DayOfWeek::Fri,
        ],
        TimeRange::try_from_str("9AM-10AM").unwrap(),
    );
    ctx.events.insert(event);

    let mut task = Task::new("t", 1.0, None, Date(start));
    task.push_subtask_with_hours(TimeRange::try_from_str("10AM-11AM").unwrap(), start, 1.0);
    ctx.tasks.insert(task);

    let day_range = TimeRange::try_from_str("8AM-12PM").unwrap();
    let free = CalendarView::free_blocks_for_date(&ctx, start, &day_range);
    assert_eq!(free.len(), 2);
    assert!((free[0].remaining_free_time - 1.0).abs() < f32::EPSILON); // 8-9
    assert!((free[1].remaining_free_time - 1.0).abs() < f32::EPSILON); // 11-12
}

#[test]
fn coalesce_merges_adjacent_free_blocks() {
    // Build overlapping/adjacent free blocks to hit coalesce logic directly.
    let start = super::sample_date();
    let day_range = TimeRange::try_from_str("8AM-12PM").unwrap();
    let base = NaiveDateTime::new(start, day_range.start);
    let blocks = vec![
        FreeTimeBlock::new(base, base + chrono::Duration::hours(1)), // 8-9
        FreeTimeBlock::new(
            base + chrono::Duration::hours(1),
            base + chrono::Duration::hours(2),
        ), // 9-10 (adjacent to first)
        FreeTimeBlock::new(
            base + chrono::Duration::hours(3),
            base + chrono::Duration::hours(4),
        ), // 11-12 (separate)
    ];

    let free = CalendarView::coalesce_free_blocks(blocks);
    assert_eq!(free.len(), 2);
    assert_eq!(free[0].remaining_free_time, 2.0); // 8-10 merged
    assert_eq!(free[1].remaining_free_time, 1.0); // 11-12
    assert!(free[0].start_time < free[0].end_time);
    assert!(free[1].start_time < free[1].end_time);
}
