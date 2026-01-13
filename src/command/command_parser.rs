use crate::arg::args::Arg;
use crate::command::command_resolver::{
    AddEntityResolver, CommandResolver, EntityActionResolver, GlobalResolver, TypeHelpResolver,
};
use crate::command::commands::CommandDyn;
use crate::errors::{Error, Result};

pub struct CommandParser {
    registry: Vec<Box<dyn CommandResolver>>,
}

impl CommandParser {
    pub fn new() -> Self {
        Self {
            registry: vec![
                Box::new(EntityActionResolver),
                Box::new(AddEntityResolver),
                Box::new(GlobalResolver),
                Box::new(TypeHelpResolver),
            ],
        }
    }

    pub fn parse<'a>(&self, command: &str, args: &'a [Arg]) -> Result<CommandDyn<'a>> {
        for r in &self.registry {
            if r.can_resolve(command) {
                return r.resolve(command, args);
            }
        }
        Err(Error::UnknownCommand(command.to_string()))
    }
}
