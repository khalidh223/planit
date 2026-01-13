// core/scheduling/overflow.rs
use crate::core::models::Task;
use crate::core::types::TaskOverflowPolicy;
use crate::errors::{Error, Result};

pub trait OverflowPolicyHandler {
    fn handle(&self, task: &mut Task, placed_any: bool) -> Result<()>;
}

pub struct AllowOverflow;
pub struct BlockOverflow;

impl OverflowPolicyHandler for AllowOverflow {
    fn handle(&self, task: &mut Task, placed_any: bool) -> Result<()> {
        if task.remaining_hours > 0.0 && placed_any {
            if let Some(last) = task.subtasks.last_mut() {
                if last.task_id == task.id {
                    last.overflow = true;
                }
            }
        }
        Ok(())
    }
}

impl OverflowPolicyHandler for BlockOverflow {
    fn handle(&self, task: &mut Task, _placed_any: bool) -> Result<()> {
        if task.remaining_hours > 0.0 {
            return Err(Error::Parse(format!(
                "Could not fully schedule task {} ('{}') on {}",
                task.id, task.name, task.date
            )));
        }
        Ok(())
    }
}

pub fn make_overflow_handler(policy: TaskOverflowPolicy) -> Box<dyn OverflowPolicyHandler> {
    match policy {
        TaskOverflowPolicy::Allow => Box::new(AllowOverflow),
        TaskOverflowPolicy::Block => Box::new(BlockOverflow),
    }
}
