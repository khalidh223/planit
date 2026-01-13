use crate::arg::args::Arg;
use crate::command::commands::{
    Command, CommandDyn, ConfigCommand, EntityCommand, LogCommand, ManCommand, ReadCommand,
    SaveCommand, ScheduleCommand,
};
use crate::core::types::{EntityActionType, EntityType, GlobalCommand, TypeHelpCommand};
use crate::errors::{Error, Result};
use crate::extensions::enums::valid_csv;

pub trait CommandResolver {
    fn can_resolve(&self, command: &str) -> bool;
    fn resolve<'a>(&self, command: &str, args: &'a [Arg]) -> Result<CommandDyn<'a>>;
}

pub struct EntityActionResolver;

impl CommandResolver for EntityActionResolver {
    fn can_resolve(&self, command: &str) -> bool {
        if let Ok(action) = EntityActionType::try_from(command) {
            matches!(action, EntityActionType::Modify | EntityActionType::Delete)
        } else {
            false
        }
    }

    fn resolve<'a>(&self, command: &str, args: &'a [Arg]) -> Result<CommandDyn<'a>> {
        let action = EntityActionType::try_from(command)?;
        let entity_type = match args.first() {
            Some(Arg::EntityType(t)) => *t,
            _ => {
                return Err(Error::Parse(
                    format!(
                        "Expected entity type as first argument. Valid entity types: {}",
                        valid_csv::<EntityType>()
                    )
                    .into(),
                ));
            }
        };
        let entity_command = EntityCommand::new(action, entity_type, args);
        if args.len() < 2 {
            return Err(Error::Parse(format!(
                "Missing argument(s).\n{}",
                entity_command.usage()
            )));
        }

        Ok(Box::new(entity_command))
    }
}

pub struct AddEntityResolver;

impl CommandResolver for AddEntityResolver {
    fn can_resolve(&self, command: &str) -> bool {
        EntityType::try_from(command).is_ok()
    }

    fn resolve<'a>(&self, command: &str, args: &'a [Arg]) -> Result<CommandDyn<'a>> {
        let entity_type = EntityType::try_from(command)?;
        Ok(Box::new(EntityCommand::new(
            EntityActionType::Add,
            entity_type,
            args,
        )))
    }
}

pub struct GlobalResolver;

impl CommandResolver for GlobalResolver {
    fn can_resolve(&self, command: &str) -> bool {
        GlobalCommand::try_from(command).is_ok()
    }

    fn resolve<'a>(&self, command: &str, args: &'a [Arg]) -> Result<CommandDyn<'a>> {
        let command_type = GlobalCommand::try_from(command)?;
        match command_type {
            GlobalCommand::Schedule => Ok(Box::new(ScheduleCommand::new(args))),
            GlobalCommand::Config => Ok(Box::new(ConfigCommand::new(args))),
            GlobalCommand::Log => Ok(Box::new(LogCommand::new(args))),
            GlobalCommand::Save => Ok(Box::new(SaveCommand::new(args))),
            GlobalCommand::Read => Ok(Box::new(ReadCommand::new(args))),
            GlobalCommand::Man => Ok(Box::new(ManCommand::new(args))),
        }
    }
}

pub struct TypeHelpResolver;

impl CommandResolver for TypeHelpResolver {
    fn can_resolve(&self, command: &str) -> bool {
        TypeHelpCommand::try_from(command).is_ok()
    }

    fn resolve<'a>(&self, command: &str, args: &'a [Arg]) -> Result<CommandDyn<'a>> {
        let command_type = TypeHelpCommand::try_from(command)?;
        Ok(Box::new(crate::command::commands::TypeHelpCommand::new(
            args,
            command_type,
        )))
    }
}
