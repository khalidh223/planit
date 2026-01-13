use crate::core::models::Task;
use crate::core::types::TaskSchedulingOrder;
use std::cmp::Ordering;

pub trait TaskOrderComparator {
    fn cmp(&self, a: &Task, b: &Task) -> Ordering;
}

// Due date asc; within same due date, LONGER remaining first; tie-break by id
pub struct ShortestTaskOrderComparator;
impl TaskOrderComparator for ShortestTaskOrderComparator {
    fn cmp(&self, a: &Task, b: &Task) -> Ordering {
        let by_due = a.date.0.cmp(&b.date.0);
        if by_due != Ordering::Equal {
            return by_due;
        }
        // longer remaining first
        let by_rem = b
            .remaining_hours
            .partial_cmp(&a.remaining_hours)
            .unwrap_or(Ordering::Equal);
        if by_rem != Ordering::Equal {
            return by_rem;
        }
        a.id.cmp(&b.id)
    }
}

// Due date asc; within same due date, SHORTER remaining first; tie-break by id
pub struct LongestTaskOrderComparator;
impl TaskOrderComparator for LongestTaskOrderComparator {
    fn cmp(&self, a: &Task, b: &Task) -> Ordering {
        let by_due = a.date.0.cmp(&b.date.0);
        if by_due != Ordering::Equal {
            return by_due;
        }
        // shorter remaining first
        let by_rem = a
            .remaining_hours
            .partial_cmp(&b.remaining_hours)
            .unwrap_or(Ordering::Equal);
        if by_rem != Ordering::Equal {
            return by_rem;
        }
        a.id.cmp(&b.id)
    }
}

// Due date asc; tie-break by id
pub struct DueDateOnlyComparator;
impl TaskOrderComparator for DueDateOnlyComparator {
    fn cmp(&self, a: &Task, b: &Task) -> Ordering {
        let by_due = a.date.0.cmp(&b.date.0);
        if by_due != Ordering::Equal {
            return by_due;
        }
        a.id.cmp(&b.id)
    }
}

pub fn make_task_order_comparator(kind: TaskSchedulingOrder) -> Box<dyn TaskOrderComparator> {
    match kind {
        TaskSchedulingOrder::ShortestTaskFirst => Box::new(ShortestTaskOrderComparator),
        TaskSchedulingOrder::LongestTaskFirst => Box::new(LongestTaskOrderComparator),
        TaskSchedulingOrder::DueOnly => Box::new(DueDateOnlyComparator),
    }
}
