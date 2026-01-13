use crate::arg::args::{Arg, AtSymbolArg, CardColorIdArg, DateArg, IntArg, NameArg};
use crate::command::entity_spec::common::{
    card_id_validator, entity_slot, id_slot, task_start_date_validator,
};
use crate::command::entity_spec::core::{
    ArgPattern, ArgSchema, ArgSlot, ArgValidator, ColumnIndexer, EntityBuilder, EntitySpec,
    PatternIdExt,
};
use crate::core::context::AppContext;
use crate::core::models::Task;
use crate::core::types::{EntityActionType, EntityType};
use crate::errors::{Error, Result};
use std::fmt;

pub struct TaskArgSchema;

impl TaskArgSchema {
    fn hours_slot() -> ArgSlot {
        ArgSlot::is_of_arg_type::<IntArg>().with_validator(|arg| match arg {
            Arg::Int(h) if *h > 0 => Ok(()),
            _ => Err(Error::Parse("Hours must be greater than 0.".into())),
        })
    }

    fn pattern_base() -> ArgPattern {
        vec![
            ArgSlot::is_of_arg_type::<NameArg>(),
            Self::hours_slot(),
            ArgSlot::is_of_arg_type::<CardColorIdArg>()
                .with_validator_ctx(card_id_validator())
                .optional(),
            ArgSlot::is_of_arg_type::<AtSymbolArg>(),
            ArgSlot::is_of_arg_type::<DateArg>().with_validator_ctx(task_start_date_validator()),
        ]
    }

    fn pattern_entity_id() -> ArgPattern {
        vec![entity_slot(EntityType::Task), id_slot()]
    }

    fn pattern_entity_first() -> ArgPattern {
        let mut v = Self::pattern_entity_id();
        v.extend(Self::pattern_base());
        v
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskPat {
    Base,
    EntityFirst,
    EntityId,
}

impl TaskPat {
    const fn usage(self) -> &'static str {
        match self {
            TaskPat::Base => {
                r#"task "<name>" <hours> [cardId] @ <date>
Required:
  name  - (string) Name of task, wrapped in single or double quotes
  hours - (int)    Number of hours to complete the task
  date  - (Date)   Due date to complete the task by. Run 'date -h' to see valid formats for date
Optional:
  cardId - (integer) Id referencing a Card for its tag and color. Must prefix with '+C'"#
            }

            TaskPat::EntityFirst => {
                r#"task <id> "<name>" <hours> [cardId] @ <date>
Required:
  id    - (int)    id of task
  name  - (string) Name of task, wrapped in single or double quotes
  hours - (int)    Number of hours to complete the task
  date  - (Date)   Due date to complete the task by. Run 'date -h' to see valid formats for date
Optional:
  cardId - (integer) Id referencing a Card for its tag and color. Must prefix with '+C'"#
            }

            TaskPat::EntityId => {
                r#"task <id>
Required:
  id    - (int)    id of task"#
            }
        }
    }
}

impl fmt::Display for TaskPat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.usage())
    }
}

impl PatternIdExt for TaskPat {
    fn pattern(&self) -> ArgPattern {
        match self {
            TaskPat::Base => TaskArgSchema::pattern_base(),
            TaskPat::EntityFirst => TaskArgSchema::pattern_entity_first(),
            TaskPat::EntityId => TaskArgSchema::pattern_entity_id(),
        }
    }
}

impl ArgSchema for TaskArgSchema {
    type PatternId = TaskPat;

    fn patterns_for(&self, action: EntityActionType) -> Vec<TaskPat> {
        match action {
            EntityActionType::Add => vec![TaskPat::Base],
            EntityActionType::Modify => vec![TaskPat::EntityFirst],
            EntityActionType::Delete => vec![TaskPat::EntityId],
        }
    }
}

pub struct TaskArgValidator;
impl ArgValidator for TaskArgValidator {
    type PatternId = TaskPat;

    fn validate(
        &self,
        _args: &[Arg],
        _action: EntityActionType,
        _pat_id: Self::PatternId,
    ) -> Result<()> {
        Ok(())
    }
}


pub struct TaskBuilder;
impl EntityBuilder<Task> for TaskBuilder {
    type PatternId = TaskPat;
    fn create(&self, args: &[Arg], pat_id: TaskPat) -> Result<Task> {
        match pat_id {
            TaskPat::Base => {
                let pattern = pat_id.pattern();
                let mut ix = ColumnIndexer::new(args, &pattern);
                Ok(Task::new(
                    ix.next::<NameArg>().clone(),
                    ix.next::<IntArg>() as f32,
                    ix.next_opt::<CardColorIdArg>(),
                    ix.advance().next::<DateArg>().clone(),
                ))
            }
            _ => Err(Error::Parse(
                "No valid ADD pattern matched for task.".into(),
            )),
        }
    }

    fn modify<'a>(
        &self,
        existing: &'a mut Task,
        args: &[Arg],
        pat_id: TaskPat,
    ) -> Result<&'a Task> {
        match pat_id {
            TaskPat::EntityFirst => {
                let pattern = pat_id.pattern();
                let mut ix = ColumnIndexer::new(args, &pattern);
                existing.modify(
                    ix.advance().advance().next::<NameArg>().clone(),
                    ix.next::<IntArg>() as f32,
                    ix.next_opt::<CardColorIdArg>(),
                    ix.advance().next::<DateArg>().clone(),
                );
                Ok(&*existing)
            }
            _ => Err(Error::Parse(
                "No valid MODIFY pattern matched for task.".into(),
            )),
        }
    }
}

pub struct TaskSpec {
    schema: TaskArgSchema,
    validator: TaskArgValidator,
    builder: TaskBuilder,
}

impl TaskSpec {
    pub fn new() -> Self {
        Self {
            schema: TaskArgSchema,
            validator: TaskArgValidator,
            builder: TaskBuilder,
        }
    }
}

impl EntitySpec<Task> for TaskSpec {
    type PatternId = TaskPat;

    fn arg_schema(&self) -> &dyn ArgSchema<PatternId = TaskPat> {
        &self.schema
    }
    fn arg_validator(&self) -> &dyn ArgValidator<PatternId = TaskPat> {
        &self.validator
    }
    fn entity_builder(&self) -> &dyn EntityBuilder<Task, PatternId = TaskPat> {
        &self.builder
    }

    fn get_mut<'a>(&self, ctx: &'a mut AppContext, id: i32) -> Result<&'a mut Task> {
        ctx.tasks.get_mut(id)
    }
}
