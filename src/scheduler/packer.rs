use crate::core::models::{FreeTimeBlock, Task};
use crate::core::types::TimeRange;
use chrono::{Duration, NaiveDate, NaiveDateTime};

pub enum PlaceStep {
    /// Task finished by carving inside the block; `leftover` is the remainder of that block (if any)
    Finished { leftover: Option<FreeTimeBlock> },
    /// The whole block was used and the task still needs more hours
    UsedWholeBlock,
}

pub enum PackOutcome {
    None,
    Partial,
    Full,
}

/// Observer to capture scheduling placements (e.g., for logging).
pub trait ScheduleObserver {
    fn task_scheduled(
        &self,
        task_id: i32,
        date: NaiveDate,
        task_total_hours: f32,
        scheduled_hours: f32,
        outcome: &PackOutcome,
    );
}

pub trait BlockPacker {
    /// Pick which free block to try next (return its index in `free`).
    fn select_block_idx(&mut self, free: &Vec<FreeTimeBlock>) -> Option<usize>;

    /// Place the task into `block` according to the packer’s rule.
    fn place_one_block(&self, task: &mut Task, date: NaiveDate, block: FreeTimeBlock) -> PlaceStep;

    /// Template Method: shared outer loop, queue mgmt, and outcome calc.
    fn pack(
        &mut self,
        task: &mut Task,
        date: NaiveDate,
        free: &mut Vec<FreeTimeBlock>,
        observer: &dyn ScheduleObserver,
    ) -> PackOutcome {
        if task.remaining_hours <= 0.0 || free.is_empty() {
            return PackOutcome::None;
        }
        let start_remaining = task.remaining_hours;

        while task.remaining_hours > 0.0 {
            let idx = match self.select_block_idx(free) {
                Some(i) => i,
                None => break, // no usable blocks
            };

            // take ownership of the chosen block
            let block = free.remove(idx);

            match self.place_one_block(task, date, block) {
                PlaceStep::Finished { leftover } => {
                    if let Some(b) = leftover {
                        free.insert(idx, b);
                    } // put remainder back near where it came from
                    break; // task completed
                }
                PlaceStep::UsedWholeBlock => {
                    // nothing to reinsert; continue loop
                }
            }
        }

        let outcome = if (task.remaining_hours - start_remaining).abs() < f32::EPSILON {
            PackOutcome::None
        } else if task.remaining_hours > 0.0 {
            PackOutcome::Partial
        } else {
            PackOutcome::Full
        };

        let scheduled_hours = (start_remaining - task.remaining_hours).max(0.0);
        let total_hours = task.hours;
        observer.task_scheduled(task.id, date, total_hours, scheduled_hours, &outcome);

        outcome
    }
}

pub struct FirstFitPacker;

impl FirstFitPacker {
    #[inline]
    fn duration_hours_dt(start: NaiveDateTime, end: NaiveDateTime) -> f32 {
        (end - start).num_seconds() as f32 / 3600.0
    }
    #[inline]
    fn time_after_hours_dt(start: NaiveDateTime, hours: f32) -> NaiveDateTime {
        start + Duration::seconds((hours * 3600.0).round() as i64)
    }
}

impl BlockPacker for FirstFitPacker {
    fn select_block_idx(&mut self, free: &Vec<FreeTimeBlock>) -> Option<usize> {
        free.iter().position(|b| b.remaining_free_time > 0.0)
    }

    fn place_one_block(
        &self,
        task: &mut Task,
        date: NaiveDate,
        mut block: FreeTimeBlock,
    ) -> PlaceStep {
        let need = task.remaining_hours;
        let cap = block.remaining_free_time;

        if need <= cap {
            let end_dt = Self::time_after_hours_dt(block.start_time, need);
            let tr = TimeRange {
                start: block.start_time.time(),
                end: end_dt.time(),
            };

            task.push_subtask_with_hours(tr, date, need);

            // compute leftover head of block (if any)
            block.start_time = end_dt;
            block.remaining_free_time = Self::duration_hours_dt(block.start_time, block.end_time);
            let leftover = if block.remaining_free_time > 0.0 {
                Some(block)
            } else {
                None
            };

            PlaceStep::Finished { leftover }
        } else {
            let tr = TimeRange {
                start: block.start_time.time(),
                end: block.end_time.time(),
            };
            task.push_subtask_with_hours(tr, date, cap);
            PlaceStep::UsedWholeBlock
        }
    }
}
