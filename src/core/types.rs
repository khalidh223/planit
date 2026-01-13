use crate::errors::{Error, Result};
use crate::extensions::enums::valid_csv;
use crate::extensions::string::ToDashSeparators;
use chrono::{Datelike, Local, NaiveDate, NaiveTime, Timelike};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::str::FromStr;
use strum::IntoEnumIterator;
use strum_macros::{AsRefStr, Display, EnumIter as EnumIterDerive, EnumString};

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString, Display, AsRefStr, EnumIterDerive)]
#[strum(ascii_case_insensitive, serialize_all = "lowercase")]
pub enum GlobalCommand {
    #[strum(serialize = "config", to_string = "config")]
    Config,
    #[strum(serialize = "schedule", to_string = "schedule")]
    Schedule,
    #[strum(serialize = "log", to_string = "log")]
    Log,
    #[strum(serialize = "save", to_string = "save")]
    Save,
    #[strum(serialize = "read", to_string = "read")]
    Read,
    #[strum(serialize = "man", to_string = "man")]
    Man,
}

impl GlobalCommand {
    pub fn try_from(s: &str) -> Result<Self> {
        Self::from_str(s).map_err(|_| {
            Error::Parse(format!(
                "Unsupported global command: '{}'. Valid global commands: {}",
                s.trim(),
                valid_csv::<GlobalCommand>()
            ))
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString, Display, AsRefStr, EnumIterDerive)]
#[strum(ascii_case_insensitive, serialize_all = "lowercase")]
pub enum TypeHelpCommand {
    #[strum(serialize = "date", to_string = "date")]
    Date,
    #[strum(serialize = "time", to_string = "time")]
    Time,
    #[strum(serialize = "colors", to_string = "colors")]
    Colors,
}

impl TypeHelpCommand {
    pub fn try_from(s: &str) -> Result<Self> {
        Self::from_str(s).map_err(|_| {
            Error::Parse(format!(
                "Unsupported type help command: '{}'. Valid type help commands: {}",
                s.trim(),
                valid_csv::<TypeHelpCommand>()
            ))
        })
    }

    pub fn usage(&self) -> String {
        match self {
            TypeHelpCommand::Date => Date::usage(),
            TypeHelpCommand::Time => TimeRange::usage(),
            TypeHelpCommand::Colors => CardColor::usage(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString, Display, AsRefStr, EnumIterDerive)]
#[strum(ascii_case_insensitive, serialize_all = "lowercase")]
pub enum EntityType {
    #[strum(serialize = "task", to_string = "task")]
    Task,
    #[strum(serialize = "event", to_string = "event")]
    Event,
    #[strum(serialize = "card", to_string = "card")]
    Card,
}
impl EntityType {
    pub fn try_from(s: &str) -> Result<Self> {
        Self::from_str(s).map_err(|_| {
            Error::Parse(format!(
                "Unsupported entity type: '{}'. Valid entity types: {}",
                s.trim(),
                valid_csv::<EntityType>()
            ))
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString, AsRefStr, EnumIterDerive, Display)]
#[strum(ascii_case_insensitive, serialize_all = "lowercase")]
pub enum EntityActionType {
    #[strum(serialize = "")]
    Add,
    #[strum(serialize = "mod")]
    Modify,
    #[strum(serialize = "del")]
    Delete,
}
impl EntityActionType {
    pub fn try_from(s: &str) -> Result<Self> {
        Self::from_str(s).map_err(|_| {
            Error::Parse(format!(
                "Unsupported action: '{}'. Valid actions: {}",
                s.trim(),
                valid_csv::<EntityActionType>()
            ))
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString, Display, AsRefStr, EnumIterDerive)]
#[strum(ascii_case_insensitive, serialize_all = "lowercase")]
pub enum DayOfWeek {
    #[strum(
        serialize = "mon",
        serialize = "monday",
        serialize = "mon.",
        serialize = "m",
        to_string = "MON"
    )]
    Mon,
    #[strum(
        serialize = "tue",
        serialize = "tuesday",
        serialize = "tue.",
        serialize = "t",
        to_string = "TUE"
    )]
    Tue,
    #[strum(
        serialize = "wed",
        serialize = "wednesday",
        serialize = "wed.",
        serialize = "w",
        to_string = "WED"
    )]
    Wed,
    #[strum(
        serialize = "thu",
        serialize = "thursday",
        serialize = "thu.",
        serialize = "th",
        to_string = "THU"
    )]
    Thu,
    #[strum(
        serialize = "fri",
        serialize = "friday",
        serialize = "fri.",
        serialize = "f",
        to_string = "FRI"
    )]
    Fri,
    #[strum(
        serialize = "sat",
        serialize = "saturday",
        serialize = "sat.",
        serialize = "sa",
        to_string = "SAT"
    )]
    Sat,
    #[strum(
        serialize = "sun",
        serialize = "sunday",
        serialize = "sun.",
        serialize = "su",
        to_string = "SUN"
    )]
    Sun,
}

impl DayOfWeek {
    pub fn try_from(s: &str) -> Result<Self> {
        Self::from_str(s).map_err(|_| {
            Error::Parse(format!(
                "Invalid day of the week: '{}'. Valid days: {}",
                s.trim(),
                valid_csv::<DayOfWeek>()
            ))
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Date(pub NaiveDate);

#[derive(Copy, Clone, Debug, EnumIterDerive, AsRefStr, EnumString)]
#[strum(ascii_case_insensitive, serialize_all = "lowercase")]
pub enum DateFormat {
    #[strum(serialize = "%Y-%m-%d", to_string = "%Y-%m-%d")]
    YmdDash,
    #[strum(serialize = "%m-%d-%Y", to_string = "%m-%d-%Y")]
    MdYDash,
    #[strum(serialize = "%Y/%m/%d", to_string = "%Y/%m/%d")]
    YmdSlash,
    #[strum(serialize = "%m/%d/%Y", to_string = "%m/%d/%Y")]
    MdYSlash,
    #[strum(serialize = "%m-%d", to_string = "%m-%d")]
    MdDash,
    #[strum(serialize = "%m/%d", to_string = "%m/%d")]
    MdSlash,
}

#[derive(Debug, Clone)]
struct DateParseSpec {
    input: String,
    date_format: DateFormat,
}

impl DateFormat {
    fn build_parse_spec(self, input: &str) -> DateParseSpec {
        let current_year = Local::now().date_naive().year();
        match self {
            DateFormat::YmdDash | DateFormat::YmdSlash => DateParseSpec {
                input: input.to_owned(),
                date_format: DateFormat::YmdDash,
            },
            DateFormat::MdYDash | DateFormat::MdYSlash => DateParseSpec {
                input: input.to_owned(),
                date_format: DateFormat::MdYDash,
            },
            DateFormat::MdDash | DateFormat::MdSlash => DateParseSpec {
                input: format!("{current_year}-{input}"),
                date_format: DateFormat::YmdDash,
            },
        }
    }
}

impl Date {
    pub fn usage() -> String {
        let today = Local::now().date_naive();
        let formats = DateFormat::iter()
            .map(|df| today.format(df.as_ref()).to_string())
            .collect::<Vec<_>>()
            .join(", ");
        format!("Supported formats: {}", formats)
    }
    fn error_message(input: &str) -> String {
        format!("Invalid date format: '{}'. {}", input, Self::usage())
    }

    pub fn try_from_str(input: &str) -> Result<Self> {
        let input = input.to_dash_separators();

        for f in DateFormat::iter() {
            let spec = f.build_parse_spec(&input);
            if let Ok(date) = NaiveDate::parse_from_str(&spec.input, spec.date_format.as_ref()) {
                return Ok(Date(date));
            }
        }

        Err(Error::Parse(Self::error_message(&input)))
    }
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.format("%Y-%m-%d"))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimeRange {
    pub start: NaiveTime,
    pub end: NaiveTime,
}

#[derive(Copy, Clone, Debug, EnumIterDerive, AsRefStr, EnumString)]
pub enum TimeFormat {
    #[strum(serialize = "%-I:%M%p")]
    HmMeridian,
    #[strum(serialize = "%-I%p")]
    HMeridian,
    #[strum(serialize = "%-I:%M")]
    Hm,
    #[strum(serialize = "%-I")]
    H,
}

impl TimeFormat {
    fn token_has_meridian(token: &str) -> bool {
        token.contains("AM") || token.contains("PM")
    }

    fn ensure_minutes(self, mut token: String) -> (String, TimeFormat) {
        let mut time_format = self;

        if !token.contains(':') {
            if let Some(idx) = token.find("AM").or_else(|| token.find("PM")) {
                token.insert_str(idx, ":00");
            } else {
                token.push_str(":00");
            }

            time_format = match self {
                TimeFormat::HMeridian | TimeFormat::H => TimeFormat::HmMeridian,
                other => other,
            };
        }

        (token, time_format)
    }

    fn build_parse_spec(self, raw_token: &str, is_start: bool) -> TimeParseSpec {
        let mut token = raw_token.trim().to_ascii_uppercase();

        if !Self::token_has_meridian(&token) {
            token.push_str(if is_start { "AM" } else { "PM" });
        }

        let (token, time_format) = self.ensure_minutes(token);

        TimeParseSpec {
            input: token,
            time_format,
        }
    }
}

#[derive(Debug, Clone)]
struct TimeParseSpec {
    input: String,
    time_format: TimeFormat,
}

impl TimeRange {
    pub fn try_from_str(s: &str) -> Result<Self> {
        let s = s.trim();
        let (start, end) = s.split_once('-').ok_or_else(|| {
            Error::Parse(format!(
                "Invalid time range format: '{}'. Expected format: '<start>-<end>'.",
                s
            ))
        })?;

        let start = Self::parse_token_as_time(start.trim(), true)?;
        let end = Self::parse_token_as_time(end.trim(), false)?;

        Self::validate(start, end)
    }

    pub fn try_from_parts(start_tok: &str, end_tok: &str) -> Result<Self> {
        let start = Self::parse_token_as_time(start_tok, true)?;
        let end = Self::parse_token_as_time(end_tok, false)?;
        Self::validate(start, end)
    }

    fn parse_token_as_time(raw: &str, is_start: bool) -> Result<NaiveTime> {
        for f in TimeFormat::iter() {
            let spec = f.build_parse_spec(raw, is_start);
            if let Ok(t) = NaiveTime::parse_from_str(&spec.input, spec.time_format.as_ref()) {
                return Ok(t);
            }
        }
        Err(Error::Parse(Self::error_message(raw)))
    }

    fn validate(start: NaiveTime, end: NaiveTime) -> Result<Self> {
        if start == end {
            return Err(Error::Parse(format!(
                "Start time '{}' cannot be the same as end time '{}'.",
                start.format(TimeFormat::HmMeridian.as_ref()),
                end.format(TimeFormat::HmMeridian.as_ref())
            )));
        }
        if start >= end {
            return Err(Error::Parse(format!(
                "Start time '{}' must be earlier than end time '{}'.",
                start.format(TimeFormat::HmMeridian.as_ref()),
                end.format(TimeFormat::HmMeridian.as_ref())
            )));
        }

        Ok(TimeRange { start, end })
    }

    pub fn usage() -> String {
        let now = Local::now().time();
        let time = NaiveTime::from_hms_opt(now.hour(), now.minute(), 0).unwrap();
        let formats = TimeFormat::iter()
            .map(|fmt| time.format(fmt.as_ref()).to_string())
            .collect::<Vec<_>>()
            .join(", ");
        format!("Supported formats: {}", formats)
    }

    fn error_message(input: &str) -> String {
        format!("Invalid time format: '{}'. {}", input, Self::usage())
    }
}
impl fmt::Display for TimeRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}-{}",
            self.start.format(TimeFormat::HmMeridian.as_ref()),
            self.end.format(TimeFormat::HmMeridian.as_ref())
        )
    }
}

impl Serialize for TimeRange {
    fn serialize<S: Serializer>(
        &self,
        serializer: S,
    ) -> std::result::Result<<S as Serializer>::Ok, <S as Serializer>::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for TimeRange {
    fn deserialize<D: Deserializer<'de>>(
        deserializer: D,
    ) -> std::result::Result<TimeRange, <D as Deserializer<'de>>::Error> {
        let s = String::deserialize(deserializer)?;
        TimeRange::try_from_str(&s).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString, Display, AsRefStr, EnumIterDerive)]
#[strum(ascii_case_insensitive)]
pub enum BoolFormat {
    #[strum(serialize = "true", serialize = "True", to_string = "True")]
    TextTrue,

    #[strum(serialize = "false", serialize = "False", to_string = "False")]
    TextFalse,
}

impl BoolFormat {
    #[inline]
    fn to_bool(self) -> bool {
        matches!(self, BoolFormat::TextTrue)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bool(pub bool);

impl Bool {
    pub fn try_from_str(s: &str) -> Result<Self> {
        match BoolFormat::from_str(s) {
            Ok(fmt) => Ok(Bool(fmt.to_bool())),
            Err(_) => Err(Error::Parse(format!(
                "Invalid string value for boolean: '{}'. Valid values: {}",
                s,
                valid_csv::<BoolFormat>()
            ))),
        }
    }
}

impl fmt::Display for Bool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", if self.0 { "True" } else { "False" })
    }
}

impl Serialize for Bool {
    fn serialize<S: Serializer>(
        &self,
        serializer: S,
    ) -> std::result::Result<<S as Serializer>::Ok, <S as Serializer>::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Bool {
    fn deserialize<D: Deserializer<'de>>(
        deserializer: D,
    ) -> std::result::Result<Bool, <D as Deserializer<'de>>::Error> {
        let b = String::deserialize(deserializer)?;
        Bool::try_from_str(&b).map_err(serde::de::Error::custom)
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    EnumString,
    Display,
    AsRefStr,
    EnumIterDerive,
    Serialize,
    Deserialize,
)]
#[strum(ascii_case_insensitive, serialize_all = "lowercase")]
#[serde(rename_all = "kebab-case")]
pub enum TaskSchedulingOrder {
    #[strum(serialize = "shortest-task-first", to_string = "shortest-task-first")]
    ShortestTaskFirst,
    #[strum(serialize = "longest-task-first", to_string = "longest-task-first")]
    LongestTaskFirst,
    #[strum(serialize = "due-only", to_string = "due-only")]
    DueOnly,
}
impl TaskSchedulingOrder {
    pub fn help(&self) -> &'static str {
        match self {
            TaskSchedulingOrder::ShortestTaskFirst => {
                "Prioritize closest due date, then longest remaining hours."
            }
            TaskSchedulingOrder::LongestTaskFirst => {
                "Prioritize closest due date, then shortest remaining hours."
            }
            TaskSchedulingOrder::DueOnly => "Prioritize by due date only.",
        }
    }

    pub fn try_from(s: &str) -> Result<Self> {
        Self::from_str(s).map_err(|_| {
            Error::Parse(format!(
                "Invalid task scheduling order: '{}'. Allowed scheduling orders: {}",
                s.trim(),
                valid_csv::<TaskSchedulingOrder>()
            ))
        })
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    EnumString,
    Display,
    AsRefStr,
    EnumIterDerive,
    Serialize,
    Deserialize,
)]
#[strum(ascii_case_insensitive, serialize_all = "lowercase")]
#[serde(rename_all = "kebab-case")]
pub enum TaskOverflowPolicy {
    #[strum(serialize = "allow", to_string = "allow")]
    Allow,
    #[strum(serialize = "block", to_string = "hard-block")]
    Block,
}

impl TaskOverflowPolicy {
    pub fn help(&self) -> &'static str {
        match self {
            TaskOverflowPolicy::Allow => {
                "Place work even if it overflows the target day; the last slice is marked overflow."
            }
            TaskOverflowPolicy::Block => {
                "Fail if the task cannot be fully scheduled within the window."
            }
        }
    }

    pub fn try_from(s: &str) -> Result<Self> {
        Self::from_str(s).map_err(|_| {
            Error::Parse(format!(
                "Invalid task overflow policy: '{}'. Allowed policies: {}",
                s.trim(),
                valid_csv::<TaskOverflowPolicy>()
            ))
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScheduledTime {
    pub date: NaiveDate,
    pub time_range: TimeRange,
}
impl ScheduledTime {
    pub fn duration_in_hours(&self) -> f32 {
        let start_dt = self.date.and_time(self.time_range.start);
        let end_dt = self.date.and_time(self.time_range.end);

        let secs = end_dt.signed_duration_since(start_dt).num_seconds();
        secs as f32 / 3600.0
    }
}
impl fmt::Display for ScheduledTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.date.format("%Y-%m-%d"), self.time_range)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString, Display, AsRefStr, EnumIterDerive)]
#[strum(ascii_case_insensitive, serialize_all = "lowercase")]
pub enum CardColor {
    #[strum(serialize = "red", to_string = "RED")]
    Red,
    #[strum(serialize = "orange", to_string = "ORANGE")]
    Orange,
    #[strum(serialize = "yellow", to_string = "YELLOW")]
    Yellow,
    #[strum(serialize = "green", to_string = "GREEN")]
    Green,
    #[strum(serialize = "light_blue", to_string = "LIGHT_BLUE")]
    LightBlue,
    #[strum(serialize = "blue", to_string = "BLUE")]
    Blue,
    #[strum(serialize = "indigo", to_string = "INDIGO")]
    Indigo,
    #[strum(serialize = "violet", to_string = "VIOLET")]
    Violet,
    #[strum(serialize = "black", to_string = "BLACK")]
    Black,
    #[strum(serialize = "light_green", to_string = "LIGHT_GREEN")]
    LightGreen,
    #[strum(serialize = "light_coral", to_string = "LIGHT_CORAL")]
    LightCoral,
}
impl CardColor {
    pub const RESET: &'static str = crate::csi!("0m");

    /// Foreground ANSI color for this card color.
    pub fn ansi_fg(self) -> &'static str {
        match self {
            CardColor::Red => crate::csi!("31m"),
            CardColor::Orange => crate::csi!("33m"),
            CardColor::Yellow => crate::csi!("33m"),
            CardColor::Green => crate::csi!("32m"),
            CardColor::LightBlue => crate::csi!("36m"),
            CardColor::Blue => crate::csi!("34m"),
            CardColor::Indigo => crate::csi!("35m"),
            CardColor::Violet => crate::csi!("35m"),
            CardColor::Black => crate::csi!("30m"),
            CardColor::LightGreen => crate::csi!("32m"),
            CardColor::LightCoral => crate::csi!("31m"),
        }
    }

    pub fn paint<S: AsRef<str>>(self, s: S) -> String {
        format!("{}{}{}", self.ansi_fg(), s.as_ref(), Self::RESET)
    }

    pub fn usage() -> String {
        let colors = valid_csv::<CardColor>();
        format!("Valid colors: {}", colors)
    }
    pub fn try_from(s: &str) -> Result<Self> {
        Self::from_str(s).map_err(|_| {
            Error::Parse(format!(
                "Invalid value for color: '{}'. {}",
                s.trim(),
                Self::usage()
            ))
        })
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    EnumString,
    Display,
    AsRefStr,
    EnumIterDerive,
    Serialize,
    Deserialize,
)]
#[strum(ascii_case_insensitive, serialize_all = "lowercase")]
pub enum Flag {
    #[strum(serialize = "-h", serialize = "-help", to_string = "-h")]
    Help,
}
