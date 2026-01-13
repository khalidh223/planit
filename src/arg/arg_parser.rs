use super::args::*;
use crate::errors::{Error, Result};

pub struct ArgParser {
    factories: Vec<Box<dyn ArgFactory>>,
}

impl ArgParser {
    pub fn new() -> Self {
        Self {
            factories: vec![
                Box::new(MultiTokenFactory::<NameArg>::new()),
                Box::new(SingleTokenFactory::<EntityTypeArg>::new()),
                Box::new(SingleTokenFactory::<AtSymbolArg>::new()),
                Box::new(SingleTokenFactory::<CardColorArg>::new()),
                Box::new(SingleTokenFactory::<FlagArg>::new()),
                Box::new(SingleTokenFactory::<BoolArg>::new()),
                Box::new(SingleTokenFactory::<IntArg>::new()),
                Box::new(MultiTokenFactory::<DaysOfWeekArg>::new()),
                Box::new(SingleTokenFactory::<TimeRangeArg>::new()),
                Box::new(SingleTokenFactory::<DateArg>::new()),
                Box::new(SingleTokenFactory::<CardColorIdArg>::new()),
            ],
        }
    }

    pub fn parse(&self, raw: &[String]) -> Result<Vec<Arg>> {
        let mut ts = TokenStream::new(raw);
        let mut out = Vec::new();

        while !ts.eof() {
            let tok = ts.peek()?.to_string();
            let mut claimed = false;

            for f in &self.factories {
                if f.can_start(&tok) {
                    out.push(f.parse(&mut ts)?);
                    claimed = true;
                    break;
                }
            }

            if !claimed {
                return Err(Error::Parse(format!(
                    "Unrecognized argument: '{}'. If this is a name/title, wrap it in quotes.",
                    tok
                )));
            }
        }
        Ok(out)
    }
}
