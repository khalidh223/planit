use std::collections::HashMap;

use crate::arg::arg_parser::ArgParser;
use crate::arg::args::{Arg, NameArg, SingleTokenArg};
use crate::errors::Result;

pub trait ArgParseStrategy {
    fn parse(&self, raw: &[String]) -> Result<Vec<Arg>>;
}

pub struct StandardArgParser {
    parser: ArgParser,
}

impl StandardArgParser {
    pub fn new() -> Self {
        Self {
            parser: ArgParser::new(),
        }
    }
}

impl ArgParseStrategy for StandardArgParser {
    fn parse(&self, raw: &[String]) -> Result<Vec<Arg>> {
        self.parser.parse(raw)
    }
}

pub struct ManArgParser;

impl ArgParseStrategy for ManArgParser {
    fn parse(&self, raw: &[String]) -> Result<Vec<Arg>> {
        if raw.is_empty() {
            return Ok(Vec::new());
        }

        let joined = raw.join(" ");
        let normalized = strip_wrapping_quotes(&joined);
        Ok(vec![Arg::Name(normalized)])
    }
}

pub struct CommandArgParser {
    default: StandardArgParser,
    overrides: HashMap<String, Box<dyn ArgParseStrategy>>,
}

impl CommandArgParser {
    pub fn new() -> Self {
        let mut overrides: HashMap<String, Box<dyn ArgParseStrategy>> = HashMap::new();
        overrides.insert("man".to_string(), Box::new(ManArgParser));
        Self {
            default: StandardArgParser::new(),
            overrides,
        }
    }

    pub fn parse(&self, command: &str, raw: &[String]) -> Result<Vec<Arg>> {
        let key = command.trim().to_ascii_lowercase();
        if let Some(parser) = self.overrides.get(&key) {
            parser.parse(raw)
        } else {
            self.default.parse(raw)
        }
    }
}

fn strip_wrapping_quotes(value: &str) -> String {
    let trimmed = value.trim();
    if NameArg::accepts(trimmed) {
        if let Ok(Arg::Name(name)) = NameArg::new(trimmed) {
            return name;
        }
    }
    trimmed.to_string()
}
