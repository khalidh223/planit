use super::{chrono::WeekdayExt, enums::valid_csv, string::ToDashSeparators};
use crate::core::types::DayOfWeek;
use chrono::Weekday;

#[test]
fn weekday_ext_maps_to_domain_enum() {
    let pairs = [
        (Weekday::Mon, DayOfWeek::Mon),
        (Weekday::Tue, DayOfWeek::Tue),
        (Weekday::Wed, DayOfWeek::Wed),
        (Weekday::Thu, DayOfWeek::Thu),
        (Weekday::Fri, DayOfWeek::Fri),
        (Weekday::Sat, DayOfWeek::Sat),
        (Weekday::Sun, DayOfWeek::Sun),
    ];
    for (weekday, expected) in pairs {
        assert_eq!(weekday.to_day_of_week(), expected);
    }
}

#[test]
fn valid_csv_lists_enum_variants_as_strings() {
    let csv = valid_csv::<DayOfWeek>();
    assert!(csv.contains("MON"));
    assert!(csv.contains("SUN"));
    assert!(csv.contains(","));
}

#[test]
fn to_dash_separators_replaces_and_trims() {
    let s = " 2025/01/02 ";
    assert_eq!(s.to_dash_separators(), "2025-01-02");

    let owned = "a/b/c".to_string();
    assert_eq!(owned.to_dash_separators(), "a-b-c");
}
