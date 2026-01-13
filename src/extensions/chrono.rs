use crate::core::types::DayOfWeek;
use chrono::Weekday;

pub trait WeekdayExt {
    fn to_day_of_week(self) -> DayOfWeek;
}

impl WeekdayExt for Weekday {
    fn to_day_of_week(self) -> DayOfWeek {
        match self {
            Weekday::Mon => DayOfWeek::Mon,
            Weekday::Tue => DayOfWeek::Tue,
            Weekday::Wed => DayOfWeek::Wed,
            Weekday::Thu => DayOfWeek::Thu,
            Weekday::Fri => DayOfWeek::Fri,
            Weekday::Sat => DayOfWeek::Sat,
            Weekday::Sun => DayOfWeek::Sun,
        }
    }
}
