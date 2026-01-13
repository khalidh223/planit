use crate::arg::args::{
    Arg, AtSymbolArg, BoolArg, CardColorArg, CardColorIdArg, DateArg, DaysOfWeekArg, EntityTypeArg,
    FlagArg, IntArg, NameArg, TimeRangeArg,
};
use crate::core::types::{
    BoolFormat, CardColor, DateFormat, DayOfWeek, EntityType, Flag, TimeFormat,
};
use crate::errors::Error;
use crate::extensions::enums::valid_csv;

pub trait ArgMatcher {
    fn matches_variant(actual: &Arg) -> bool;
    fn expected_error(provided: &Arg) -> Error;
}

impl ArgMatcher for NameArg {
    fn matches_variant(actual: &Arg) -> bool {
        matches!(actual, Arg::Name(_))
    }
    fn expected_error(provided: &Arg) -> Error {
        Error::Parse(format!("Expected a quoted name, got {}", provided))
    }
}

impl ArgMatcher for IntArg {
    fn matches_variant(a: &Arg) -> bool {
        matches!(a, Arg::Int(_))
    }
    fn expected_error(provided: &Arg) -> Error {
        Error::Parse(format!("Expected an integer, got {:?}", provided))
    }
}

impl ArgMatcher for BoolArg {
    fn matches_variant(actual: &Arg) -> bool {
        matches!(actual, Arg::Bool(_))
    }
    fn expected_error(provided: &Arg) -> Error {
        Error::Parse(format!(
            "Expected a boolean, got {:?}. Valid booleans: {}",
            provided,
            valid_csv::<BoolFormat>()
        ))
    }
}

impl ArgMatcher for FlagArg {
    fn matches_variant(actual: &Arg) -> bool {
        matches!(actual, Arg::Flag(_))
    }
    fn expected_error(provided: &Arg) -> Error {
        Error::Parse(format!(
            "Expected a flag, got {:?}. Valid flags: {}",
            provided,
            valid_csv::<Flag>()
        ))
    }
}

impl ArgMatcher for CardColorArg {
    fn matches_variant(actual: &Arg) -> bool {
        matches!(actual, Arg::CardColor(_))
    }
    fn expected_error(provided: &Arg) -> Error {
        Error::Parse(format!(
            "Expected a card color, got {:?}. Valid colors: {}",
            provided,
            valid_csv::<CardColor>()
        ))
    }
}

impl ArgMatcher for CardColorIdArg {
    fn matches_variant(actual: &Arg) -> bool {
        matches!(actual, Arg::CardColorId(_))
    }
    fn expected_error(provided: &Arg) -> Error {
        Error::Parse(format!(
            "Expected a card color id in the format '+C<integer>', got {:?}.",
            provided
        ))
    }
}

impl ArgMatcher for AtSymbolArg {
    fn matches_variant(actual: &Arg) -> bool {
        matches!(actual, Arg::AtSymbol)
    }
    fn expected_error(provided: &Arg) -> Error {
        Error::Parse(format!("Expected an @ symbol, got {:?}.", provided))
    }
}

impl ArgMatcher for DaysOfWeekArg {
    fn matches_variant(actual: &Arg) -> bool {
        matches!(actual, Arg::DaysOfWeek(_))
    }
    fn expected_error(provided: &Arg) -> Error {
        Error::Parse(format!(
            "Expected a comma separated list of days in the week, got {:?}. Valid days: {}",
            provided,
            valid_csv::<DayOfWeek>()
        ))
    }
}

impl ArgMatcher for TimeRangeArg {
    fn matches_variant(actual: &Arg) -> bool {
        matches!(actual, Arg::TimeRange(_))
    }
    fn expected_error(provided: &Arg) -> Error {
        Error::Parse(format!(
            "Expected a time range in the format <start>-<end>, got {:?}. Supported time formats: {}",
            provided,
            valid_csv::<TimeFormat>()
        ))
    }
}

impl ArgMatcher for DateArg {
    fn matches_variant(actual: &Arg) -> bool {
        matches!(actual, Arg::Date(_))
    }
    fn expected_error(provided: &Arg) -> Error {
        Error::Parse(format!(
            "Expected a valid date, got {:?}. Valid date formats: {}",
            provided,
            valid_csv::<DateFormat>()
        ))
    }
}

impl ArgMatcher for EntityTypeArg {
    fn matches_variant(actual: &Arg) -> bool {
        matches!(actual, Arg::EntityType(_))
    }
    fn expected_error(provided: &Arg) -> Error {
        Error::Parse(format!(
            "Expected an entity type, got {:?}. Valid entity types {}",
            provided,
            valid_csv::<EntityType>()
        ))
    }
}
