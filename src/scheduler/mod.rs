use crate::core::context::AppContext;
use crate::core::repository::Sort;
use crate::core::types::{TaskOverflowPolicy, TaskSchedulingOrder, TimeRange};
use crate::errors::Result;
use crate::logging::{LogTarget, Logger};
use crate::scheduler::calendar_view::CalendarView;
use crate::scheduler::comparator::make_task_order_comparator;
use crate::scheduler::overflow::make_overflow_handler;
use crate::scheduler::packer::{BlockPacker, FirstFitPacker, PackOutcome, ScheduleObserver};
use crate::ui::display_manager::DisplayManager;
use chrono::{Local, NaiveDate};

mod calendar_view;
mod comparator;
mod overflow;
mod packer;
#[cfg(test)]
mod tests;

struct LoggerObserver {
    logger: Logger,
}

impl ScheduleObserver for LoggerObserver {
    fn task_scheduled(
        &self,
        task_id: i32,
        date: NaiveDate,
        task_total_hours: f32,
        scheduled_hours: f32,
        outcome: &PackOutcome,
    ) {
        match outcome {
            PackOutcome::Full => {
                if (scheduled_hours - task_total_hours).abs() < f32::EPSILON {
                    self.logger.info(
                        format!(
                            "Task with id {} completely scheduled on date {}",
                            task_id, date
                        ),
                        LogTarget::FileOnly,
                    );
                } else {
                    self.logger.info(
                        format!(
                            "All remaining hours of task with id {} has been scheduled on date {} ({} hours)",
                            task_id, date, scheduled_hours
                        ),
                        LogTarget::FileOnly,
                    );
                }
            }
            PackOutcome::Partial => {
                self.logger.info(
                    format!(
                        "Task with id {} scheduled on date {} ({} hours)",
                        task_id, date, scheduled_hours
                    ),
                    LogTarget::FileOnly,
                );
            }
            PackOutcome::None => { /* nothing placed */ }
        }
    }
}

pub struct ScheduleManager<'a> {
    ctx: &'a mut AppContext,

    // Precomputed / configured at construction
    daywin: TimeRange,
    order: TaskSchedulingOrder,
    policy: TaskOverflowPolicy,
    packer: Box<dyn BlockPacker>,
    days_to_plan: u32,
    observer: LoggerObserver,
}

impl<'a> ScheduleManager<'a> {
    pub fn new(ctx: &'a mut AppContext) -> Self {
        // Read once from config
        let daywin = ctx.config.range().clone();
        let order = *ctx.config.task_scheduling_order();
        let policy = *ctx.config.task_overflow_policy();

        // Choose default packer; you can make this configurable, too.
        let packer: Box<dyn BlockPacker> = Box::new(FirstFitPacker);

        // Decide planning window length here (or read from config)
        let days_to_plan = 7;

        let observer = LoggerObserver {
            logger: ctx.logger.clone(),
        };

        Self {
            ctx,
            daywin,
            order,
            policy,
            packer,
            days_to_plan,
            observer,
        }
    }

    /// Template Method: reset → iterate days → schedule tasks → apply overflow policy
    pub fn compute_schedule(&mut self) -> Result<()> {
        self.ctx
            .logger
            .info("Starting scheduling...", LogTarget::FileOnly);
        self.reset_tasks();

        // These are cheap to build each run and don't borrow self.ctx
        let cmp = make_task_order_comparator(self.order);
        let overflow = make_overflow_handler(self.policy);

        let start_date = self
            .ctx
            .config
            .schedule_start_date()
            .unwrap_or_else(|| Local::now().date_naive());

        // Precompute planning days once.
        let days: Vec<_> = CalendarView::new()
            .with_start_date(start_date)
            .with_days(self.days_to_plan)
            .days();

        for date in &days {
            let mut free_blocks =
                CalendarView::free_blocks_for_date(&*self.ctx, *date, &self.daywin);

            for event in self.ctx.events.values(Sort::Unordered) {
                if event.is_active_on_date(*date) {
                    let msg = format!("Event with id {} scheduled on date {}", event.id, date);
                    self.ctx.logger.info(msg, LogTarget::FileOnly);
                }
            }

            self.ctx
                .tasks
                .query_mut()
                .r#where(|t| *date <= t.date.0 && t.remaining_hours > 0.0)
                .order_with(|a, b| cmp.cmp(a, b))
                .for_each_mut(|task| {
                    let outcome = if task.remaining_hours <= 0.0 {
                        PackOutcome::None
                    } else {
                        self.packer
                            .pack(task, *date, &mut free_blocks, &self.observer)
                    };

                    match outcome {
                        PackOutcome::None => { /* nothing placed */ }
                        PackOutcome::Partial | PackOutcome::Full => {
                            let _ = overflow.handle(task, true);
                        }
                    }
                });
        }

        let dm = DisplayManager::new();
        dm.display_schedule_for_days(&days, &self.ctx.tasks, &self.ctx.events, &self.ctx.cards);
        self.ctx
            .logger
            .info("Finished scheduling.", LogTarget::ConsoleAndFile);

        Ok(())
    }

    fn reset_tasks(&mut self) {
        for t in self.ctx.tasks.values_mut() {
            t.subtasks.clear();
            t.remaining_hours = t.hours;
        }
    }
}
