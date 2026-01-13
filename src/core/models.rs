use crate::core::types::{CardColor, Date, DayOfWeek, TimeRange};
use crate::extensions::chrono::WeekdayExt;
use chrono::Datelike;
use chrono::{NaiveDate, NaiveDateTime};
use std::fmt;

pub trait BaseEntity {
    fn id(&self) -> i32;
    fn set_id(&mut self, id: i32);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubTask {
    pub task_id: i32,
    pub date: NaiveDate,
    pub time_range: TimeRange,
    pub overflow: bool,
}
impl SubTask {
    pub fn hours(&self) -> f32 {
        let start_dt = NaiveDateTime::new(self.date, self.time_range.start);
        let end_dt = NaiveDateTime::new(self.date, self.time_range.end);
        (end_dt - start_dt).num_seconds() as f32 / 3600.0
    }
}

impl fmt::Display for SubTask {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.date.format("%Y-%m-%d"), self.time_range)
    }
}

#[derive(Debug, Clone)]
pub struct Task {
    pub id: i32,
    pub name: String,
    pub hours: f32,
    pub date: Date,
    pub card_id: Option<i32>,
    pub subtasks: Vec<SubTask>,
    pub remaining_hours: f32,
}
impl Task {
    pub fn new(name: impl Into<String>, hours: f32, card_id: Option<i32>, date: Date) -> Self {
        let h = hours.max(0.0);
        Self {
            id: 1,
            name: name.into(),
            hours: h,
            date,
            card_id,
            subtasks: Vec::new(),
            remaining_hours: h,
        }
    }

    pub fn modify(
        &mut self,
        name: impl Into<String>,
        hours: f32,
        card_id: Option<i32>,
        date: Date,
    ) -> &Self {
        let h = hours.max(0.0);
        self.name = name.into();
        self.hours = h;
        self.date = date;
        self.card_id = card_id;
        self.remaining_hours = h;
        self.subtasks.clear();
        self
    }

    pub fn push_subtask_with_hours(&mut self, time_range: TimeRange, date: NaiveDate, hours: f32) {
        let apply = hours.max(0.0).min(self.remaining_hours);

        self.subtasks.push(SubTask {
            task_id: self.id,
            date,
            time_range,
            overflow: false,
        });

        self.remaining_hours -= apply;
    }
}
impl BaseEntity for Task {
    fn id(&self) -> i32 {
        self.id
    }
    fn set_id(&mut self, id: i32) {
        self.id = id;
    }
}
impl fmt::Display for Task {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let subtasks = if self.subtasks.is_empty() {
            "Not Scheduled".to_string()
        } else {
            self.subtasks
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        };
        write!(
            f,
            "Task(id={}, name='{}', hours={}, date={}, card_id={:?}, subtasks={})",
            self.id, self.name, self.hours, self.date, self.card_id, subtasks
        )
    }
}

// =====
// Card
// =====

#[derive(Debug, Clone)]
pub struct Card {
    pub id: i32,
    pub name: String,
    pub color: CardColor,
}
impl Card {
    pub fn new(name: impl Into<String>, color: CardColor) -> Self {
        Self {
            id: 1,
            name: name.into(),
            color,
        }
    }

    pub fn modify(&mut self, name: impl Into<String>, color: CardColor) -> &Self {
        self.name = name.into();
        self.color = color;
        self
    }
}
impl BaseEntity for Card {
    fn id(&self) -> i32 {
        self.id
    }
    fn set_id(&mut self, id: i32) {
        self.id = id;
    }
}
impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Card(id={}, name='{}', color={})",
            self.id, self.name, self.color
        )
    }
}

#[derive(Debug, Clone)]
pub struct Event {
    pub id: i32,
    pub name: String,
    pub days: Vec<DayOfWeek>,
    pub time_range: TimeRange,
    pub recurring: bool,
    pub card_id: Option<i32>,
}

impl Event {
    pub fn new(
        recurring: bool,
        name: impl Into<String>,
        card_id: Option<i32>,
        days: Vec<DayOfWeek>,
        time_range: TimeRange,
    ) -> Self {
        Self {
            id: 1,
            recurring,
            name: name.into(),
            days,
            time_range,
            card_id,
        }
    }

    pub fn modify(
        &mut self,
        recurring: bool,
        name: impl Into<String>,
        card_id: Option<i32>,
        days: Vec<DayOfWeek>,
        time_range: TimeRange,
    ) -> &Self {
        self.recurring = recurring;
        self.name = name.into();
        self.days = days;
        self.time_range = time_range;
        self.card_id = card_id;
        self
    }

    pub fn hours(&self) -> f32 {
        let base = NaiveDate::from_ymd_opt(1970, 1, 1).unwrap();
        let start = NaiveDateTime::new(base, self.time_range.start);
        let end = NaiveDateTime::new(base, self.time_range.end);
        (end - start).num_seconds() as f32 / 3600.0
    }

    pub fn is_active_on_date(&self, target_date: NaiveDate) -> bool {
        let day_of_week = target_date.weekday().to_day_of_week();
        self.days.iter().any(|d| *d == day_of_week)
    }
}

impl BaseEntity for Event {
    fn id(&self) -> i32 {
        self.id
    }
    fn set_id(&mut self, id: i32) {
        self.id = id;
    }
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let days_str = self
            .days
            .iter()
            .map(|d| d.to_string()) // "MON", "TUE", ...
            .collect::<Vec<_>>()
            .join(", ");
        write!(
            f,
            "Event(id={}, name='{}', date=[{}], time_range={}, recurring={}, card_id={:?})",
            self.id, self.name, days_str, self.time_range, self.recurring, self.card_id
        )
    }
}

#[derive(Debug, Clone)]
pub struct FreeTimeBlock {
    pub start_time: NaiveDateTime,
    pub end_time: NaiveDateTime,
    pub remaining_free_time: f32,
}
impl FreeTimeBlock {
    pub fn new(start_time: NaiveDateTime, end_time: NaiveDateTime) -> Self {
        let hrs = (end_time - start_time).num_seconds() as f32 / 3600.0;
        Self {
            start_time,
            end_time,
            remaining_free_time: hrs.max(0.0),
        }
    }
}
