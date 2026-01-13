use crate::core::context::AppContext;
use crate::core::models::FreeTimeBlock;
use crate::core::repository::Sort;
use crate::core::types::TimeRange;
use chrono::{Duration, Local, NaiveDate, NaiveDateTime};

pub struct CalendarView {
    start: NaiveDate,
    days: u32,
}

impl CalendarView {
    /// Default: 7 days starting today.
    pub fn new() -> Self {
        Self {
            start: Local::now().date_naive(),
            days: 7,
        }
    }

    pub fn with_start_date(mut self, start: NaiveDate) -> Self {
        self.start = start;
        self
    }

    /// Plan for `days` days.
    pub fn with_days(mut self, days: u32) -> Self {
        self.days = days;
        self
    }

    /// Ordered list of planning dates.
    pub fn days(&self) -> Vec<NaiveDate> {
        (0..self.days)
            .map(|offset| self.start + Duration::days(offset as i64))
            .collect()
    }

    /// Build free blocks for `date` from day window minus events and existing subtasks.
    pub fn free_blocks_for_date(
        ctx: &AppContext,
        date: NaiveDate,
        day_range: &TimeRange,
    ) -> Vec<FreeTimeBlock> {
        let mut free = vec![FreeTimeBlock::new(
            NaiveDateTime::new(date, day_range.start),
            NaiveDateTime::new(date, day_range.end),
        )];

        // subtract events
        let events = ctx
            .events
            .query()
            .r#where(|event| event.is_active_on_date(date))
            .collect();
        for event in events {
            free = Self::subtract_busy_from_free(date, &free, &event.time_range);
        }

        // subtract already scheduled subtasks
        for t in ctx.tasks.values(Sort::Unordered) {
            for st in &t.subtasks {
                if st.date == date {
                    free = Self::subtract_busy_from_free(date, &free, &st.time_range);
                }
            }
        }

        Self::coalesce_free_blocks(free)
    }

    // -------- internals (unchanged helpers) --------

    fn subtract_busy_from_free(
        date: NaiveDate,
        free: &[FreeTimeBlock],
        busy: &TimeRange,
    ) -> Vec<FreeTimeBlock> {
        let busy_start = NaiveDateTime::new(date, busy.start);
        let busy_end = NaiveDateTime::new(date, busy.end);
        let mut out = Vec::new();
        for fb in free {
            if busy_end <= fb.start_time || busy_start >= fb.end_time {
                out.push(fb.clone());
                continue;
            }
            if busy_start > fb.start_time {
                out.push(FreeTimeBlock::new(fb.start_time, busy_start));
            }
            if busy_end < fb.end_time {
                out.push(FreeTimeBlock::new(busy_end, fb.end_time));
            }
        }
        out
    }

    #[cfg_attr(test, allow(dead_code))]
    pub(crate) fn coalesce_free_blocks(mut v: Vec<FreeTimeBlock>) -> Vec<FreeTimeBlock> {
        if v.is_empty() {
            return v;
        }
        v.sort_by_key(|b| (b.start_time, b.end_time));
        let mut out = Vec::with_capacity(v.len());
        let mut cur = v[0].clone();
        for b in v.into_iter().skip(1) {
            if b.start_time <= cur.end_time {
                if b.end_time > cur.end_time {
                    cur.end_time = b.end_time;
                    cur.remaining_free_time = Self::duration_hours_dt(cur.start_time, cur.end_time);
                }
            } else {
                out.push(cur);
                cur = b;
            }
        }
        out.push(cur);
        out
    }

    fn duration_hours_dt(start: NaiveDateTime, end: NaiveDateTime) -> f32 {
        (end - start).num_seconds() as f32 / 3600.0
    }
}
