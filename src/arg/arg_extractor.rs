use crate::arg::arg_matcher::ArgMatcher;
use crate::arg::args::{
    Arg, AtSymbolArg, BoolArg, CardColorArg, CardColorIdArg, DateArg, DaysOfWeekArg, EntityTypeArg,
    FlagArg, IntArg, NameArg, TimeRangeArg,
};
use crate::core::types::{Bool, CardColor, Date, DayOfWeek, EntityType, Flag, TimeRange};

pub trait ArgExtractor<'a>: ArgMatcher {
    type Out;

    fn try_extract(actual: &'a Arg) -> Option<Self::Out>;
}

#[inline]
pub fn extract_at<'a, E: ArgExtractor<'a>>(args: &'a [Arg], idx: usize) -> E::Out {
    let a = &args[idx];
    if let Some(v) = E::try_extract(a) {
        return v;
    }
    unreachable!("{}", E::expected_error(a));
}

impl<'a> ArgExtractor<'a> for NameArg {
    type Out = &'a String;
    fn try_extract(a: &'a Arg) -> Option<Self::Out> {
        if !NameArg::matches_variant(a) {
            return None;
        }
        match a {
            Arg::Name(s) => Some(s),
            _ => None,
        }
    }
}

impl<'a> ArgExtractor<'a> for IntArg {
    type Out = i32;
    fn try_extract(a: &'a Arg) -> Option<Self::Out> {
        if !IntArg::matches_variant(a) {
            return None;
        }
        match a {
            Arg::Int(v) => Some(*v),
            _ => None,
        }
    }
}

impl<'a> ArgExtractor<'a> for AtSymbolArg {
    type Out = ();
    fn try_extract(a: &'a Arg) -> Option<Self::Out> {
        if !AtSymbolArg::matches_variant(a) {
            return None;
        }
        Some(())
    }
}

impl<'a> ArgExtractor<'a> for DateArg {
    type Out = &'a Date;
    fn try_extract(a: &'a Arg) -> Option<Self::Out> {
        if !DateArg::matches_variant(a) {
            return None;
        }
        match a {
            Arg::Date(d) => Some(d),
            _ => None,
        }
    }
}

impl<'a> ArgExtractor<'a> for TimeRangeArg {
    type Out = &'a TimeRange;
    fn try_extract(a: &'a Arg) -> Option<Self::Out> {
        if !TimeRangeArg::matches_variant(a) {
            return None;
        }
        match a {
            Arg::TimeRange(t) => Some(t),
            _ => None,
        }
    }
}

impl<'a> ArgExtractor<'a> for DaysOfWeekArg {
    type Out = &'a Vec<DayOfWeek>;
    fn try_extract(a: &'a Arg) -> Option<Self::Out> {
        if !DaysOfWeekArg::matches_variant(a) {
            return None;
        }
        match a {
            Arg::DaysOfWeek(v) => Some(v),
            _ => None,
        }
    }
}

impl<'a> ArgExtractor<'a> for BoolArg {
    type Out = Bool; // Copy
    fn try_extract(a: &'a Arg) -> Option<Self::Out> {
        if !BoolArg::matches_variant(a) {
            return None;
        }
        match a {
            Arg::Bool(b) => Some(*b),
            _ => None,
        }
    }
}

impl<'a> ArgExtractor<'a> for EntityTypeArg {
    type Out = EntityType; // Copy
    fn try_extract(a: &'a Arg) -> Option<Self::Out> {
        if !EntityTypeArg::matches_variant(a) {
            return None;
        }
        match a {
            Arg::EntityType(t) => Some(*t),
            _ => None,
        }
    }
}

impl<'a> ArgExtractor<'a> for CardColorArg {
    type Out = CardColor; // Copy
    fn try_extract(a: &'a Arg) -> Option<Self::Out> {
        if !CardColorArg::matches_variant(a) {
            return None;
        }
        match a {
            Arg::CardColor(c) => Some(*c),
            _ => None,
        }
    }
}

impl<'a> ArgExtractor<'a> for FlagArg {
    type Out = Flag; // Copy
    fn try_extract(a: &'a Arg) -> Option<Self::Out> {
        if !FlagArg::matches_variant(a) {
            return None;
        }
        match a {
            Arg::Flag(f) => Some(*f),
            _ => None,
        }
    }
}

impl<'a> ArgExtractor<'a> for CardColorIdArg {
    type Out = i32;
    fn try_extract(a: &'a Arg) -> Option<Self::Out> {
        if !CardColorIdArg::matches_variant(a) {
            return None;
        }
        match a {
            Arg::CardColorId(c) => Some(*c),
            _ => None,
        }
    }
}
