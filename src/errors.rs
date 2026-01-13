use thiserror::Error;

// Re-export a simple Result alias used across the crate.
pub type Result<T> = std::result::Result<T, Error>;

use crate::core::types::TimeRange;

/// Domain-specific error set mirroring your Python exceptions.
#[derive(Error, Debug)]
pub enum Error {
    // ---- Parsing & Routing --------------------------------------------------
    /// Arg/lex/semantic arg problems (ArgParser, EntitySpec matching, etc.)
    #[error("Parse error: {0}")]
    Parse(String),

    /// No resolver or command match (CommandParser).
    #[error("Unknown command: {0}")]
    UnknownCommand(String),

    // ---- Scheduling / Domain -----------------------------------------------
    /// Raised when at least one task cannot be fully scheduled.
    #[error(
        "Task overflow: '{task_name}' still has {remaining_hours:.1} hour(s) after scheduling."
    )]
    TaskOverflow {
        task_name: String,
        remaining_hours: f32,
    },

    /// Raised when an event falls outside the configured daily range.
    #[error("Event '{event_name}' time {event_time} is outside daily range {daily_range}.")]
    EventOutsideOfDailyRange {
        event_name: String,
        event_time: TimeRange,
        daily_range: TimeRange,
    },

    // ---- Config -------------------------------------------------------------
    /// Any issue initializing/reading config (file missing, invalid JSON, etc.)
    #[error("Config error: {0}")]
    Config(String),

    /// Specific missing config item (used by ConfigValue-like accessors).
    #[error("Missing configuration item: {item}")]
    ConfigItemMissing { item: &'static str },

    // ---- Plumbing / Wrappers ------------------------------------------------
    /// Generic domain error when you want to bubble a message without a new variant.
    #[error("{0}")]
    Domain(String),

    /// IO passthrough (read/write files, etc.)
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Serde JSON passthrough (config JSON decode/encode, etc.)
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

// ----------------------- Convenience constructors ----------------------------

impl Error {
    /// Helper to create a arg error from any displayable value.
    pub fn parse<S: Into<String>>(msg: S) -> Self {
        Error::Parse(msg.into())
    }
    /// Helper to create a generic config error.
    pub fn config<S: Into<String>>(msg: S) -> Self {
        Error::Config(msg.into())
    }
    /// Helper for unknown command.
    pub fn unknown<S: Into<String>>(cmd: S) -> Self {
        Error::UnknownCommand(cmd.into())
    }
}

// ----------------------- Small result helpers --------------------------------

/// Map an `Option<T>` into `Result<T, Error::Parse>` with a custom message.
/// Useful when extracting required fields from Arg field maps.
pub fn require_parse<T, S: Into<String>>(opt: Option<T>, msg: S) -> Result<T> {
    opt.ok_or_else(|| Error::Parse(msg.into()))
}

/// Map an `Option<T>` into `Result<T, Error::ConfigItemMissing>` with a static key.
pub fn require_config_item<T>(opt: Option<T>, item: &'static str) -> Result<T> {
    opt.ok_or_else(|| Error::ConfigItemMissing { item })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::TimeRange;

    #[test]
    fn parse_constructor_wraps_message() {
        let err = Error::parse("bad args");
        match err {
            Error::Parse(msg) => assert_eq!(msg, "bad args"),
            other => panic!("expected parse error, got {other:?}"),
        }
    }

    #[test]
    fn config_constructor_wraps_message() {
        let err = Error::config("config missing");
        match err {
            Error::Config(msg) => assert_eq!(msg, "config missing"),
            other => panic!("expected config error, got {other:?}"),
        }
    }

    #[test]
    fn unknown_constructor_wraps_message() {
        let err = Error::unknown("noop");
        match err {
            Error::UnknownCommand(msg) => assert_eq!(msg, "noop"),
            other => panic!("expected unknown command error, got {other:?}"),
        }
    }

    #[test]
    fn require_parse_returns_value_when_present() {
        let value = require_parse(Some(4), "missing").unwrap();
        assert_eq!(value, 4);
    }

    #[test]
    fn require_parse_errors_with_message_when_missing() {
        let err = require_parse::<i32, _>(None, "missing").unwrap_err();
        match err {
            Error::Parse(msg) => assert_eq!(msg, "missing"),
            other => panic!("expected parse error, got {other:?}"),
        }
    }

    #[test]
    fn require_config_item_errors_with_key() {
        let err = require_config_item::<i32>(None, "range").unwrap_err();
        match err {
            Error::ConfigItemMissing { item } => assert_eq!(item, "range"),
            other => panic!("expected config item missing error, got {other:?}"),
        }
    }

    #[test]
    fn task_overflow_error_formats_message() {
        let remaining = 1.25;
        let err = Error::TaskOverflow {
            task_name: "alpha".to_string(),
            remaining_hours: remaining,
        };
        let expected = format!(
            "Task overflow: 'alpha' still has {:.1} hour(s) after scheduling.",
            remaining
        );
        assert_eq!(err.to_string(), expected);
    }

    #[test]
    fn event_outside_range_formats_message() {
        let event_time = TimeRange::try_from_str("8AM-9AM").unwrap();
        let daily_range = TimeRange::try_from_str("10AM-11AM").unwrap();
        let err = Error::EventOutsideOfDailyRange {
            event_name: "standup".to_string(),
            event_time,
            daily_range,
        };
        let expected =
            "Event 'standup' time 8:00AM-9:00AM is outside daily range 10:00AM-11:00AM."
                .to_string();
        assert_eq!(err.to_string(), expected);
    }

    #[test]
    fn domain_error_displays_raw_message() {
        let err = Error::Domain("oops".to_string());
        assert_eq!(err.to_string(), "oops");
    }

    #[test]
    fn io_error_formats_message() {
        let raw = std::io::Error::new(std::io::ErrorKind::Other, "disk");
        let err = Error::from(raw);
        assert_eq!(err.to_string(), "I/O error: disk");
    }

    #[test]
    fn json_error_formats_message() {
        let raw = serde_json::from_str::<serde_json::Value>("not-json").unwrap_err();
        let expected = format!("JSON error: {}", raw);
        let err = Error::from(raw);
        assert_eq!(err.to_string(), expected);
    }
}
