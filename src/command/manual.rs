use std::fmt::Display;
use std::str::FromStr;

use strum::IntoEnumIterator;
use strum_macros::{AsRefStr, Display as DisplayDerive, EnumIter as EnumIterDerive, EnumString};

use crate::command::entity_spec::{
    card::CardSpec,
    core::{EntitySpec, PatternIdExt},
    event::EventSpec,
    task::TaskSpec,
};
use crate::core::types::{EntityActionType, EntityType, GlobalCommand, TypeHelpCommand};
use crate::errors::{Error, Result};
use crate::extensions::enums::valid_csv;

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString, DisplayDerive, AsRefStr, EnumIterDerive)]
#[strum(ascii_case_insensitive, serialize_all = "lowercase")]
pub enum ManualTopic {
    General,
    Task,
    Event,
    Card,
    Config,
    Schedule,
    Log,
    Save,
    Read,
    Man,
    Date,
    Time,
    Colors,
}

impl ManualTopic {
    pub fn try_from(input: &str) -> Result<Self> {
        Self::from_str(input).map_err(|_| {
            Error::Parse(format!(
                "Unsupported manual topic: '{}'. Valid topics: {}",
                input.trim(),
                valid_csv::<ManualTopic>()
            ))
        })
    }
}

#[derive(Debug, Clone)]
pub struct ManualSection {
    title: String,
    body: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ManualPage {
    name: String,
    summary: String,
    sections: Vec<ManualSection>,
}

impl ManualPage {
    pub fn render(&self) -> String {
        let mut out = String::new();
        self.write_section(
            "NAME",
            &[format!("{} - {}", self.name, self.summary)],
            &mut out,
        );
        for section in &self.sections {
            self.write_section(&section.title, &section.body, &mut out);
        }
        out.trim_end().to_string()
    }

    fn write_section(&self, title: &str, lines: &[String], out: &mut String) {
        out.push_str(&title.to_uppercase());
        out.push('\n');
        for line in lines {
            out.push_str("  ");
            out.push_str(line);
            out.push('\n');
        }
        out.push('\n');
    }
}

pub struct ManualPageBuilder {
    name: String,
    summary: String,
    sections: Vec<ManualSection>,
}

impl ManualPageBuilder {
    pub fn new(name: impl Into<String>, summary: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            summary: summary.into(),
            sections: Vec::new(),
        }
    }

    pub fn section(mut self, title: &str, body: Vec<String>) -> Self {
        self.sections.push(ManualSection {
            title: title.to_string(),
            body,
        });
        self
    }

    pub fn build(self) -> ManualPage {
        ManualPage {
            name: self.name,
            summary: self.summary,
            sections: self.sections,
        }
    }
}

pub struct ManualCatalog;

impl ManualCatalog {
    pub fn new() -> Self {
        Self
    }

    pub fn page_for(&self, topic: Option<&str>) -> Result<ManualPage> {
        let topic = match topic {
            None => ManualTopic::General,
            Some(name) => ManualTopic::try_from(name)?,
        };
        Ok(self.build_page(topic))
    }

    pub fn topics(&self) -> Vec<String> {
        ManualTopic::iter().map(|t| t.to_string()).collect()
    }

    fn build_page(&self, topic: ManualTopic) -> ManualPage {
        match topic {
            ManualTopic::General => self.general_page(),
            ManualTopic::Task => self.entity_page(
                EntityType::Task,
                "Create and manage tasks with due dates.",
                vec![
                    "Tasks track required hours and a due date.".to_string(),
                    "They are scheduled before the due date when possible.".to_string(),
                ],
                TaskSpec::new(),
            ),
            ManualTopic::Event => self.entity_page(
                EntityType::Event,
                "Create and manage events on a schedule.",
                vec![
                    "Events reserve time blocks on one or more days.".to_string(),
                    "Recurring events can specify multiple days.".to_string(),
                ],
                EventSpec::new(),
            ),
            ManualTopic::Card => self.entity_page(
                EntityType::Card,
                "Create and manage colored tags for tasks and events.",
                vec![
                    "Cards provide colored labels for tasks and events.".to_string(),
                    "Reference cards with +C<id> when adding entities.".to_string(),
                ],
                CardSpec::new(),
            ),
            ManualTopic::Config => self.simple_page(
                "config",
                "View or edit configuration values.",
                vec!["config".to_string()],
                vec![
                    "Opens an interactive configuration editor.".to_string(),
                    "Press enter to accept a selection and update a value.".to_string(),
                ],
            ),
            ManualTopic::Schedule => self.simple_page(
                "schedule",
                "Generate a schedule based on current tasks and events.",
                vec!["schedule".to_string()],
                vec!["Uses the current config to build a schedule.".to_string()],
            ),
            ManualTopic::Log => self.simple_page(
                "log",
                "Print the current session log to the console.",
                vec!["log".to_string()],
                vec![
                    "Shows the session log file contents if it exists.".to_string(),
                    "Does not create a log file when one is missing.".to_string(),
                ],
            ),
            ManualTopic::Save => self.simple_page(
                "save",
                "Save tasks, events, and cards to a schedule file.",
                vec!["save \"<name>\"".to_string()],
                vec!["Writes to schedules/<name>.json.".to_string()],
            ),
            ManualTopic::Read => self.simple_page(
                "read",
                "Load tasks, events, and cards from a schedule file.",
                vec!["read \"<path>\"".to_string()],
                vec!["Loads entities into the current session.".to_string()],
            ),
            ManualTopic::Man => self.simple_page(
                "man",
                "Show manual pages for commands and topics.",
                vec!["man [topic]".to_string()],
                vec![
                    format!("Topics: {}", self.topics().join(", ")),
                    "Use 'man' with no topic for the general manual.".to_string(),
                ],
            ),
            ManualTopic::Date => self.type_help_page(TypeHelpCommand::Date),
            ManualTopic::Time => self.type_help_page(TypeHelpCommand::Time),
            ManualTopic::Colors => self.type_help_page(TypeHelpCommand::Colors),
        }
    }

    fn general_page(&self) -> ManualPage {
        ManualPageBuilder::new("planit", "Personal scheduling CLI.")
            .section("SYNOPSIS", vec!["<command> [args]".to_string()])
            .section("COMMANDS", general_command_lines())
            .section(
                "TOPICS",
                vec![
                    "Use 'man <topic>' for command-specific details.".to_string(),
                    format!("Available topics: {}", self.topics().join(", ")),
                ],
            )
            .build()
    }

    fn simple_page(
        &self,
        name: &str,
        summary: &str,
        synopsis: Vec<String>,
        description: Vec<String>,
    ) -> ManualPage {
        ManualPageBuilder::new(name, summary)
            .section("SYNOPSIS", synopsis)
            .section("DESCRIPTION", description)
            .build()
    }

    fn type_help_page(&self, kind: TypeHelpCommand) -> ManualPage {
        ManualPageBuilder::new(kind.to_string(), "Type helper command.")
            .section("SYNOPSIS", vec![kind.to_string()])
            .section("DESCRIPTION", vec![kind.usage()])
            .build()
    }

    fn entity_page<E, P>(
        &self,
        entity: EntityType,
        summary: &str,
        description: Vec<String>,
        spec: impl EntitySpec<E, PatternId = P>,
    ) -> ManualPage
    where
        P: Copy + Eq + PatternIdExt + Display,
    {
        let mut usage = Vec::new();
        for (idx, group) in [
            self.usage_lines(EntityActionType::Add, &spec),
            self.usage_lines(EntityActionType::Modify, &spec),
            self.usage_lines(EntityActionType::Delete, &spec),
        ]
        .into_iter()
        .enumerate()
        {
            usage.extend(group);
            if idx < 2 {
                usage.push(String::new());
            }
        }

        ManualPageBuilder::new(entity.to_string(), summary)
            .section("SYNOPSIS", usage)
            .section("DESCRIPTION", description)
            .section(
                "SEE ALSO",
                vec![
                    GlobalCommand::Schedule.to_string(),
                    GlobalCommand::Config.to_string(),
                    GlobalCommand::Man.to_string(),
                ],
            )
            .build()
    }

    fn usage_lines<E, P>(
        &self,
        action: EntityActionType,
        spec: &impl EntitySpec<E, PatternId = P>,
    ) -> Vec<String>
    where
        P: Copy + Eq + PatternIdExt + Display,
    {
        let prefix = match action {
            EntityActionType::Add => None,
            _ => Some(action.to_string()),
        };
        spec.arg_schema()
            .patterns_for(action)
            .into_iter()
            .map(|pid| match &prefix {
                Some(p) => format!("{p} {pid}"),
                None => pid.to_string(),
            })
            .collect()
    }
}

fn general_command_lines() -> Vec<String> {
    vec![
        "task \"<name>\" <hours> [cardId] @ <date>  # Add a task".to_string(),
        "event <recurring> \"<name>\" [cardId] @ [days] <timeRange>  # Add an event".to_string(),
        "card \"<name>\" <color>                   # Add a card".to_string(),
        "mod <entity> <id> ...                     # Modify an entity".to_string(),
        "del <entity> <id>                         # Delete an entity".to_string(),
        "schedule                                 # Build the schedule".to_string(),
        "config                                   # View or edit config".to_string(),
        "save \"<name>\"                           # Save to schedules/<name>.json".to_string(),
        "read \"<path>\"                           # Load from a saved schedule file".to_string(),
        "log                                      # Print the session log".to_string(),
        "man [topic]                              # Show manual pages".to_string(),
        "date | time | colors                     # Type helper commands".to_string(),
    ]
}
