use crate::extensions::enums::valid_csv;
use std::fmt;
use std::marker::PhantomData;

use crate::core::types::{Bool, CardColor, Date, DayOfWeek, EntityType, Flag, TimeRange};
use crate::errors::{Error, Result};

#[derive(Debug, Clone)]
pub enum Arg {
    Flag(Flag),
    CardColor(CardColor),
    CardColorId(i32),
    Int(i32),
    AtSymbol,
    Bool(Bool),
    DaysOfWeek(Vec<DayOfWeek>),
    TimeRange(TimeRange),
    Date(Date),
    Name(String),
    EntityType(EntityType),
}

fn fmt_seq<T: fmt::Display>(f: &mut fmt::Formatter<'_>, items: &[T]) -> fmt::Result {
    let mut first = true;
    for item in items {
        if !first {
            write!(f, ", ")?;
        }
        write!(f, "{item}")?;
        first = false;
    }
    Ok(())
}

impl fmt::Display for Arg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Arg::Flag(x) => write!(f, "{x}"),
            Arg::CardColorId(x) => write!(f, "+C{x}"),
            Arg::CardColor(x) => write!(f, "{x}"),
            Arg::Int(x) => write!(f, "{x}"),
            Arg::AtSymbol => write!(f, "@"),
            Arg::Bool(x) => write!(f, "{x}"),
            Arg::DaysOfWeek(xs) => fmt_seq(f, xs),
            Arg::TimeRange(x) => write!(f, "{x}"),
            Arg::Date(x) => write!(f, "{x}"),
            Arg::Name(x) => write!(f, "\"{x}\""),
            Arg::EntityType(x) => write!(f, "{x}"),
        }
    }
}

impl Arg {
    pub fn to_tokens(&self) -> Vec<String> {
        let rendered = self.to_string();
        if rendered.contains(char::is_whitespace)
            && !rendered.starts_with('"')
            && !rendered.starts_with('\'')
        {
            rendered.split_whitespace().map(|tok| tok.to_string()).collect()
        } else {
            vec![rendered]
        }
    }
}

#[derive(Debug, Clone)]
pub struct TokenStream {
    toks: Vec<String>,
    i: usize,
}
impl TokenStream {
    pub fn new(raw: &[String]) -> Self {
        Self {
            toks: raw.to_vec(),
            i: 0,
        }
    }
    pub fn eof(&self) -> bool {
        self.i >= self.toks.len()
    }
    pub fn peek(&self) -> Result<&str> {
        self.toks
            .get(self.i)
            .map(|s| s.as_str())
            .ok_or_else(|| Error::Parse("EOF".into()))
    }
    pub fn next(&mut self) -> Result<String> {
        let s = self.peek()?.to_string();
        self.i += 1;
        Ok(s)
    }
}


pub trait SingleTokenArg {
    fn accepts(tok: &str) -> bool;
    fn new(tok: &str) -> Result<Arg>;
}

pub trait MultiTokenArg: SingleTokenArg {
    fn starts_sequence(tok: &str) -> bool;
}

pub trait ArgFactory {
    fn can_start(&self, tok: &str) -> bool;
    fn parse(&self, ts: &mut TokenStream) -> Result<Arg>;
}

pub struct SingleTokenFactory<A: SingleTokenArg>(PhantomData<A>);
impl<A: SingleTokenArg> SingleTokenFactory<A> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}
impl<A: SingleTokenArg> ArgFactory for SingleTokenFactory<A> {
    fn can_start(&self, tok: &str) -> bool {
        A::accepts(tok)
    }
    fn parse(&self, ts: &mut TokenStream) -> Result<Arg> {
        let tok = ts.next()?;
        A::new(&tok)
    }
}

pub struct MultiTokenFactory<A: MultiTokenArg>(PhantomData<A>);
impl<A: MultiTokenArg> MultiTokenFactory<A> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}
impl<A: MultiTokenArg> ArgFactory for MultiTokenFactory<A> {
    fn can_start(&self, tok: &str) -> bool {
        A::starts_sequence(tok)
    }
    fn parse(&self, ts: &mut TokenStream) -> Result<Arg> {
        let mut buf: Vec<String> = vec![ts.next()?];
        loop {
            let joined = buf.join(" ");
            if A::accepts(&joined) {
                return A::new(&joined);
            }
            if ts.eof() {
                return A::new(&joined);
            }
            buf.push(ts.next()?);
        }
    }
}

pub struct NameArg;
impl MultiTokenArg for NameArg {
    fn starts_sequence(value: &str) -> bool {
        !value.is_empty() && matches!(value.as_bytes()[0], b'\'' | b'"')
    }
}

impl SingleTokenArg for NameArg {
    fn accepts(value: &str) -> bool {
        if value.len() < 2 {
            return false;
        }
        let q = value.as_bytes()[0] as char;
        if q != '\'' && q != '"' {
            return false;
        }
        if !value.ends_with(q) {
            return false;
        }
        value[1..value.len() - 1].len() > 0
    }
    fn new(value: &str) -> Result<Arg> {
        if !Self::accepts(value) {
            return Err(Error::Parse(
                "Name must contain text wrapped in single or double quotes.".into(),
            ));
        }
        Ok(Arg::Name(value[1..value.len() - 1].to_string()))
    }
}

pub struct DaysOfWeekArg;

impl MultiTokenArg for DaysOfWeekArg {
    fn starts_sequence(tok: &str) -> bool {
        // Start only if first token looks like a day (optionally with trailing comma).
        let t = tok.trim().trim_end_matches(',');
        DayOfWeek::try_from(t).is_ok()
    }
}

impl SingleTokenArg for DaysOfWeekArg {
    fn accepts(value: &str) -> bool {
        let mut saw_any = false;
        for seg in value.split(',') {
            let d = seg.trim();
            if d.is_empty() {
                return false;
            }
            if DayOfWeek::try_from(d).is_err() {
                return false;
            }
            saw_any = true;
        }
        saw_any
    }

    fn new(value: &str) -> Result<Arg> {
        if !Self::accepts(value) {
            return Err(Error::Parse(format!(
                "Invalid list of days: '{}'. Valid possible days: {}",
                value,
                valid_csv::<DayOfWeek>(),
            )));
        }

        let mut out = Vec::new();
        for seg in value.split(',') {
            let d = seg.trim();
            out.push(DayOfWeek::try_from(d)?);
        }
        Ok(Arg::DaysOfWeek(out))
    }
}

pub struct EntityTypeArg;
impl SingleTokenArg for EntityTypeArg {
    fn accepts(value: &str) -> bool {
        EntityType::try_from(value).is_ok()
    }
    fn new(value: &str) -> Result<Arg> {
        Ok(Arg::EntityType(EntityType::try_from(value)?))
    }
}

pub struct AtSymbolArg;
impl SingleTokenArg for AtSymbolArg {
    fn accepts(value: &str) -> bool {
        value == "@"
    }
    fn new(_: &str) -> Result<Arg> {
        Ok(Arg::AtSymbol)
    }
}

pub struct CardColorArg;
impl SingleTokenArg for CardColorArg {
    fn accepts(value: &str) -> bool {
        CardColor::try_from(value).is_ok()
    }
    fn new(value: &str) -> Result<Arg> {
        Ok(Arg::CardColor(CardColor::try_from(value)?))
    }
}

pub struct CardColorIdArg;

impl SingleTokenArg for CardColorIdArg {
    fn accepts(value: &str) -> bool {
        match value.strip_prefix("+C") {
            Some(rest) => !rest.is_empty() && rest.chars().all(|c| c.is_ascii_digit()),
            None => false,
        }
    }

    fn new(value: &str) -> Result<Arg> {
        if !Self::accepts(value) {
            return Err(Error::Parse(format!(
                "Invalid card color id: '{}'. Expected format '+C<number>' (e.g., +C3).",
                value
            )));
        }

        let n: i32 = value[2..].parse().map_err(|_| {
            Error::Parse(format!(
                "Invalid number in '{}'. Expected an integer after +C.",
                value
            ))
        })?;
        Ok(Arg::CardColorId(n))
    }
}

pub struct FlagArg;
impl SingleTokenArg for FlagArg {
    fn accepts(value: &str) -> bool {
        Flag::try_from(value).is_ok()
    }
    fn new(value: &str) -> Result<Arg> {
        Ok(Arg::Flag(Flag::try_from(value).map_err(|_| {
            Error::Parse(format!(
                "Invalid flag: {}. Valid flags: {}",
                value,
                valid_csv::<Flag>()
            ))
        })?))
    }
}

pub struct BoolArg;
impl SingleTokenArg for BoolArg {
    fn accepts(value: &str) -> bool {
        Bool::try_from_str(value).is_ok()
    }
    fn new(value: &str) -> Result<Arg> {
        Ok(Arg::Bool(Bool::try_from_str(value)?))
    }
}

pub struct IntArg;
impl SingleTokenArg for IntArg {
    fn accepts(value: &str) -> bool {
        value.chars().all(|c| c.is_ascii_digit())
    }
    fn new(value: &str) -> Result<Arg> {
        value
            .parse::<i32>()
            .map(Arg::Int)
            .map_err(|_| Error::Parse(format!("Expected an integer, got '{}'", value)))
    }
}

pub struct TimeRangeArg;
impl SingleTokenArg for TimeRangeArg {
    fn accepts(value: &str) -> bool {
        TimeRange::try_from_str(value).is_ok()
    }
    fn new(value: &str) -> Result<Arg> {
        Ok(Arg::TimeRange(TimeRange::try_from_str(value)?))
    }
}

pub struct DateArg;
impl SingleTokenArg for DateArg {
    fn accepts(value: &str) -> bool {
        Date::try_from_str(value).is_ok()
    }
    fn new(value: &str) -> Result<Arg> {
        Ok(Arg::Date(Date::try_from_str(value)?))
    }
}
