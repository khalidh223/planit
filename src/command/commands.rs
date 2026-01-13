use crate::arg::arg_extractor::extract_at;
use crate::arg::args::Arg;
use crate::arg::args::IntArg;
use crate::command::entity_spec::{
    card::CardSpec, core::EntitySpec, event::EventSpec, task::TaskSpec,
};
use crate::command::manual::ManualCatalog;
use crate::command::policies::flag_policy::{FlagDecision, FlagPolicy, HelpAtIdx};
use crate::core::context::AppContext;
use crate::core::persist::{load_state, save_state};
use crate::core::types::{EntityActionType, EntityType};
use crate::errors::Error::Parse;
use crate::errors::Result;
use crate::logging::LogTarget;
use crate::prompter::flows::config_edit::ConfigEditFlow;
use crate::prompter::prompter::Prompter;
use crate::scheduler::ScheduleManager;
use crate::ui::display_manager::DisplayManager;

pub struct CommandCore<'a> {
    pub args: &'a [Arg],
    pub flag_policy: FlagPolicy,
}
impl<'a> CommandCore<'a> {
    pub fn new(args: &'a [Arg], flag_policy: FlagPolicy) -> Self {
        Self { args, flag_policy }
    }
}

mod sealed {
    use super::CommandCore;

    pub trait Sealed<'a> {
        fn core(&self) -> &CommandCore<'a>;
    }
}

pub trait Command<'a>: sealed::Sealed<'a> {
    fn usage(&self) -> String;
    fn perform(&self, ctx: &mut AppContext) -> Result<()>;

    fn execute(&self, ctx: &mut AppContext) -> Result<()> {
        let core = self.core(); // available because Sealed is a supertrait (but not public)
        match core.flag_policy.evaluate(core.args) {
            FlagDecision::ShortCircuitUsage => {
                ctx.logger.info(self.usage(), LogTarget::ConsoleOnly);
                Ok(())
            }
            FlagDecision::ShortCircuitMsg(msg) => {
                ctx.logger.info(msg, LogTarget::ConsoleOnly);
                Ok(())
            }
            FlagDecision::Continue => self.perform(ctx),
            FlagDecision::Error(e) => Err(e),
        }
    }
}

pub type CommandDyn<'a> = Box<dyn Command<'a> + 'a>;

pub struct EntityCommand<'a> {
    core: CommandCore<'a>,
    action: EntityActionType,
    entity_type: EntityType,
}

impl<'a> EntityCommand<'a> {
    pub fn new(action: EntityActionType, entity_type: EntityType, args: &'a [Arg]) -> Self {
        let help_idx = match action {
            EntityActionType::Add => 0,
            EntityActionType::Modify | EntityActionType::Delete => 1,
        };
        let policy = FlagPolicy::new(vec![Box::new(HelpAtIdx(help_idx))]);
        Self {
            core: CommandCore::new(args, policy),
            action,
            entity_type,
        }
    }

    fn handle_add(&self, ctx: &mut AppContext) -> Result<()> {
        if self.core.args.is_empty() {
            DisplayManager::new().display_entities_for(
                self.entity_type,
                &ctx.tasks,
                &ctx.events,
                &ctx.cards,
            );
            return Ok(());
        }
        match self.entity_type {
            EntityType::Task => {
                let task = TaskSpec::new().create(ctx, self.core.args)?;
                let stored = ctx.tasks.insert(task);
                ctx.logger.info(
                    format!("Added task with id {}: {}", stored.id, stored),
                    LogTarget::ConsoleAndFile,
                );
            }
            EntityType::Event => {
                let event = EventSpec::new().create(ctx, self.core.args)?;
                let stored = ctx.events.insert(event);
                ctx.logger.info(
                    format!("Added event with id {}: {}", stored.id, stored),
                    LogTarget::ConsoleAndFile,
                );
            }
            EntityType::Card => {
                let card = CardSpec::new().create(ctx, self.core.args)?;
                let stored = ctx.cards.insert(card);
                ctx.logger.info(
                    format!("Added card with id {}: {}", stored.id, stored),
                    LogTarget::ConsoleAndFile,
                );
            }
        }
        Ok(())
    }

    fn handle_modify(&self, ctx: &mut AppContext) -> Result<()> {
        let id = extract_at::<IntArg>(self.core.args, 1);
        let msg = match self.entity_type {
            EntityType::Task => {
                let updated = TaskSpec::new().modify(ctx, self.core.args, id)?;
                format!("Modified task with id {}: {}", id, updated)
            }
            EntityType::Event => {
                let updated = EventSpec::new().modify(ctx, self.core.args, id)?;
                format!("Modified event with id {}: {}", id, updated)
            }
            EntityType::Card => {
                let updated = CardSpec::new().modify(ctx, self.core.args, id)?;
                format!("Modified card with id {}: {}", id, updated)
            }
        };

        ctx.logger.info(msg, LogTarget::ConsoleAndFile);
        Ok(())
    }

    fn handle_delete(&self, ctx: &mut AppContext) -> Result<()> {
        match self.entity_type {
            EntityType::Task => match TaskSpec::new().can_delete(ctx, self.core.args) {
                Err(Parse(msg)) => return Err(Parse(msg.into())),
                Ok(_) => {
                    let id = extract_at::<IntArg>(self.core.args, 1);
                    ctx.tasks.delete(id)?;
                    ctx.logger.info(
                        format!("Deleted task with id {}.", id),
                        LogTarget::ConsoleAndFile,
                    );
                }
                _ => {}
            },
            EntityType::Event => match EventSpec::new().can_delete(ctx, self.core.args) {
                Err(Parse(msg)) => return Err(Parse(msg.into())),
                Ok(_) => {
                    let id = extract_at::<IntArg>(self.core.args, 1);
                    ctx.events.delete(id)?;
                    ctx.logger.info(
                        format!("Deleted event with id {}.", id),
                        LogTarget::ConsoleAndFile,
                    );
                }
                _ => {}
            },
            EntityType::Card => match CardSpec::new().can_delete(ctx, self.core.args) {
                Err(Parse(msg)) => return Err(Parse(msg.into())),
                Ok(_) => {
                    let id = extract_at::<IntArg>(self.core.args, 1);
                    ctx.cards.delete(id)?;
                    ctx.logger.info(
                        format!("Deleted card with id {}.", id),
                        LogTarget::ConsoleAndFile,
                    );
                }
                _ => {}
            },
        }
        Ok(())
    }
}

impl<'a> sealed::Sealed<'a> for EntityCommand<'a> {
    fn core(&self) -> &CommandCore<'a> {
        &self.core
    }
}

impl<'a> Command<'a> for EntityCommand<'a> {
    fn usage(&self) -> String {
        fn join<T: std::fmt::Display>(act: EntityActionType, et: EntityType, pids: &[T]) -> String {
            if pids.is_empty() {
                return format!("{act} {et}: (no patterns)");
            }
            let mut s = String::new();
            for pid in pids {
                s.push_str(&format!("Usage: {} {}\n", act, pid));
            }
            s
        }
        match self.entity_type {
            EntityType::Task => {
                let spec = TaskSpec::new();
                join(
                    self.action,
                    self.entity_type,
                    &spec.arg_schema().patterns_for(self.action),
                )
            }
            EntityType::Event => {
                let spec = EventSpec::new();
                join(
                    self.action,
                    self.entity_type,
                    &spec.arg_schema().patterns_for(self.action),
                )
            }
            EntityType::Card => {
                let spec = CardSpec::new();
                join(
                    self.action,
                    self.entity_type,
                    &spec.arg_schema().patterns_for(self.action),
                )
            }
        }
    }

    fn perform(&self, ctx: &mut AppContext) -> Result<()> {
        match self.action {
            EntityActionType::Add => self.handle_add(ctx),
            EntityActionType::Modify => self.handle_modify(ctx),
            EntityActionType::Delete => self.handle_delete(ctx),
        }
    }
}

pub struct ConfigCommand<'a> {
    core: CommandCore<'a>,
}

impl<'a> ConfigCommand<'a> {
    pub fn new(args: &'a [Arg]) -> Self {
        let policy = FlagPolicy::new(vec![Box::new(HelpAtIdx(0))]);
        Self {
            core: CommandCore::new(args, policy),
        }
    }

    fn edit(&self, ctx: &mut AppContext) -> Result<()> {
        let prompter = Prompter::new();
        let flow = ConfigEditFlow::new(ctx);
        prompter.run(flow, true)
    }
}

impl<'a> sealed::Sealed<'a> for ConfigCommand<'a> {
    fn core(&self) -> &CommandCore<'a> {
        &self.core
    }
}

impl<'a> Command<'a> for ConfigCommand<'a> {
    fn usage(&self) -> String {
        "config   # View and edit configuration".into()
    }
    fn perform(&self, ctx: &mut AppContext) -> Result<()> {
        self.edit(ctx)
    }
}

pub struct ScheduleCommand<'a> {
    core: CommandCore<'a>,
}

impl<'a> ScheduleCommand<'a> {
    pub fn new(args: &'a [Arg]) -> Self {
        let policy = FlagPolicy::new(vec![Box::new(HelpAtIdx(0))]);
        Self {
            core: CommandCore::new(args, policy),
        }
    }
}

impl<'a> sealed::Sealed<'a> for ScheduleCommand<'a> {
    fn core(&self) -> &CommandCore<'a> {
        &self.core
    }
}

impl<'a> Command<'a> for ScheduleCommand<'a> {
    fn usage(&self) -> String {
        "schedule      # Schedule tasks".into()
    }
    fn perform(&self, ctx: &mut AppContext) -> Result<()> {
        let mut sched = ScheduleManager::new(ctx);
        sched.compute_schedule()?;
        Ok(())
    }
}

pub struct LogCommand<'a> {
    core: CommandCore<'a>,
}

impl<'a> LogCommand<'a> {
    pub fn new(args: &'a [Arg]) -> Self {
        let policy = FlagPolicy::new(vec![Box::new(HelpAtIdx(0))]);
        Self {
            core: CommandCore::new(args, policy),
        }
    }
}

impl<'a> sealed::Sealed<'a> for LogCommand<'a> {
    fn core(&self) -> &CommandCore<'a> {
        &self.core
    }
}

impl<'a> Command<'a> for LogCommand<'a> {
    fn usage(&self) -> String {
        "log          # Print current session log to console".into()
    }

    fn perform(&self, ctx: &mut AppContext) -> Result<()> {
        match ctx.logger.log_path() {
            None => {
                println!("No logs");
            }
            Some(path) => match std::fs::read_to_string(&path) {
                Ok(contents) => print!("{contents}"),
                Err(err) => {
                    eprintln!("Unable to read log file: {err}");
                }
            },
        }
        Ok(())
    }
}

pub struct ManCommand<'a> {
    core: CommandCore<'a>,
}

impl<'a> ManCommand<'a> {
    pub fn new(args: &'a [Arg]) -> Self {
        let policy = FlagPolicy::new(vec![Box::new(HelpAtIdx(0))]);
        Self {
            core: CommandCore::new(args, policy),
        }
    }

    fn topic_arg(&self) -> Result<Option<String>> {
        match self.core.args.len() {
            0 => Ok(None),
            1 => match &self.core.args[0] {
                Arg::Name(name) => Ok(Some(name.clone())),
                Arg::EntityType(entity) => Ok(Some(entity.to_string())),
                other => Err(Parse(format!(
                    "Unsupported manual topic: {}. Usage: man [topic]",
                    other
                ))),
            },
            _ => Err(Parse(
                "Expected at most one topic. Usage: man [topic]".into(),
            )),
        }
    }
}

impl<'a> sealed::Sealed<'a> for ManCommand<'a> {
    fn core(&self) -> &CommandCore<'a> {
        &self.core
    }
}

impl<'a> Command<'a> for ManCommand<'a> {
    fn usage(&self) -> String {
        "man [topic]  # Show manual pages".into()
    }

    fn perform(&self, ctx: &mut AppContext) -> Result<()> {
        let topic = self.topic_arg()?;
        let page = ManualCatalog::new().page_for(topic.as_deref())?;
        ctx.logger.info(page.render(), LogTarget::ConsoleOnly);
        Ok(())
    }
}

pub struct SaveCommand<'a> {
    core: CommandCore<'a>,
}

impl<'a> SaveCommand<'a> {
    pub fn new(args: &'a [Arg]) -> Self {
        let policy = FlagPolicy::new(vec![Box::new(HelpAtIdx(0))]);
        Self {
            core: CommandCore::new(args, policy),
        }
    }
}

impl<'a> sealed::Sealed<'a> for SaveCommand<'a> {
    fn core(&self) -> &CommandCore<'a> {
        &self.core
    }
}

impl<'a> Command<'a> for SaveCommand<'a> {
    fn usage(&self) -> String {
        "save <name>   # Save state to schedules/<name>.json".into()
    }
    fn perform(&self, ctx: &mut AppContext) -> Result<()> {
        let name = if let Some(Arg::Name(n)) = self.core.args.get(0) {
            n.clone()
        } else {
            return Err(Parse("Expected file name. Usage: save \"<name>\"".into()));
        };

        let mut path = ctx.schedules_dir.join(name);
        if path.extension().is_none() {
            path.set_extension("json");
        }

        let saved = save_state(&ctx.tasks, &ctx.events, &ctx.cards, &path)?;
        ctx.logger.info(
            format!("Saved state to {}", saved.display()),
            LogTarget::ConsoleOnly,
        );
        Ok(())
    }
}

pub struct ReadCommand<'a> {
    core: CommandCore<'a>,
}

impl<'a> ReadCommand<'a> {
    pub fn new(args: &'a [Arg]) -> Self {
        let policy = FlagPolicy::new(vec![Box::new(HelpAtIdx(0))]);
        Self {
            core: CommandCore::new(args, policy),
        }
    }
}

impl<'a> sealed::Sealed<'a> for ReadCommand<'a> {
    fn core(&self) -> &CommandCore<'a> {
        &self.core
    }
}

impl<'a> Command<'a> for ReadCommand<'a> {
    fn usage(&self) -> String {
        "read <path>   # Load state from a saved schedule file".into()
    }
    fn perform(&self, ctx: &mut AppContext) -> Result<()> {
        let path = if let Some(Arg::Name(n)) = self.core.args.get(0) {
            let candidate = std::path::PathBuf::from(n.clone());
            if candidate.is_relative() && candidate.parent().is_none() {
                ctx.schedules_dir.join(candidate)
            } else {
                candidate
            }
        } else {
            return Err(Parse("Expected file path. Usage: read \"<path>\"".into()));
        };

        load_state(ctx, &path)?;
        ctx.logger.info(
            format!("Loaded state from {}", path.display()),
            LogTarget::ConsoleOnly,
        );
        Ok(())
    }
}

pub struct TypeHelpCommand<'a> {
    core: CommandCore<'a>,
    command_type: crate::core::types::TypeHelpCommand,
}

impl<'a> TypeHelpCommand<'a> {
    pub fn new(args: &'a [Arg], command_type: crate::core::types::TypeHelpCommand) -> Self {
        let policy = FlagPolicy::new(vec![Box::new(HelpAtIdx(0))]);
        Self {
            core: CommandCore::new(args, policy),
            command_type,
        }
    }
}

impl<'a> sealed::Sealed<'a> for TypeHelpCommand<'a> {
    fn core(&self) -> &CommandCore<'a> {
        &self.core
    }
}

impl<'a> Command<'a> for TypeHelpCommand<'a> {
    fn usage(&self) -> String {
        self.command_type.usage().into()
    }
    fn perform(&self, ctx: &mut AppContext) -> Result<()> {
        ctx.logger
            .info(self.command_type.usage(), LogTarget::ConsoleOnly);
        Ok(())
    }
}
