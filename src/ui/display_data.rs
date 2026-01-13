use crate::core::models::{Card, Event, Task};
use crate::core::repository::{Repository, Sort};
use chrono::{NaiveDate, NaiveTime};

#[derive(Debug, Clone)]
pub struct ScheduleSection {
    pub title: String,
    pub rows: Vec<Vec<String>>, // already ordered by time
}

#[derive(Debug, Default, Clone)]
pub struct DisplayDataBuilder;

impl DisplayDataBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn task_rows(
        &self,
        tasks: &Repository<Task>,
        cards: &Repository<Card>,
    ) -> Vec<Vec<String>> {
        tasks
            .values(Sort::IdAsc)
            .into_iter()
            .map(|t| {
                let card_opt: Option<&Card> = t.card_id.and_then(|id| cards.get(id).ok());
                vec![
                    paint_opt(card_opt, &t.id.to_string()),
                    paint_opt(card_opt, t.name.as_str()),
                    paint_opt(card_opt, card_opt.map(|c| c.name.as_str()).unwrap_or("-")),
                    paint_opt(card_opt, &format!("{:.2}", t.hours)),
                    paint_opt(card_opt, &t.date.to_string()),
                ]
            })
            .collect()
    }

    pub fn event_rows(
        &self,
        events: &Repository<Event>,
        cards: &Repository<Card>,
    ) -> Vec<Vec<String>> {
        events
            .values(Sort::IdAsc)
            .into_iter()
            .map(|e| {
                let card_opt = e.card_id.and_then(|id| cards.get(id).ok());
                let days = if e.days.is_empty() {
                    "-".to_string()
                } else {
                    e.days
                        .iter()
                        .map(|d| d.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                };
                vec![
                    paint_opt(card_opt, &e.id.to_string()),
                    paint_opt(card_opt, e.name.as_str()),
                    paint_opt(card_opt, card_opt.map(|c| c.name.as_str()).unwrap_or("-")),
                    paint_opt(card_opt, &e.time_range.to_string()),
                    paint_opt(card_opt, days.as_str()),
                    paint_opt(card_opt, &e.recurring.to_string().to_uppercase()),
                ]
            })
            .collect()
    }

    pub fn card_rows(&self, cards: &Repository<Card>) -> Vec<Vec<String>> {
        cards
            .values(Sort::IdAsc)
            .into_iter()
            .map(|c| {
                vec![
                    c.color.paint(c.id.to_string()),
                    c.color.paint(&c.name),
                    c.color.paint(c.color.to_string()),
                ]
            })
            .collect()
    }

    pub fn build_schedule_sections(
        &self,
        dates: &[NaiveDate],
        tasks: &Repository<Task>,
        events: &Repository<Event>,
        cards: &Repository<Card>,
    ) -> Vec<ScheduleSection> {
        let mut sections: Vec<ScheduleSection> = Vec::with_capacity(dates.len());

        for date in dates {
            let mut rows: Vec<(NaiveTime, Vec<String>)> = Vec::new();

            for t in tasks.values(Sort::IdAsc) {
                let card_opt: Option<&Card> = t.card_id.and_then(|id| cards.get(id).ok());
                for st in &t.subtasks {
                    if st.date == *date {
                        rows.push((
                            st.time_range.start,
                            vec![
                                paint_opt(card_opt, &format!("T.ID: {}", t.id)),
                                paint_opt(card_opt, t.name.as_str()),
                                paint_opt(
                                    card_opt,
                                    card_opt.map(|c| c.name.as_str()).unwrap_or("-"),
                                ),
                                paint_opt(card_opt, &format!("{:.2}", st.hours())),
                                paint_opt(card_opt, &st.time_range.to_string()),
                            ],
                        ));
                    }
                }
            }

            for e in events.values(Sort::IdAsc) {
                if e.is_active_on_date(*date) {
                    let card_opt: Option<&Card> = e.card_id.and_then(|id| cards.get(id).ok());
                    rows.push((
                        e.time_range.start,
                        vec![
                            paint_opt(card_opt, &format!("E.ID: {}", e.id)),
                            paint_opt(card_opt, e.name.as_str()),
                            paint_opt(card_opt, card_opt.map(|c| c.name.as_str()).unwrap_or("-")),
                            paint_opt(card_opt, &format!("{:.2}", e.hours())),
                            paint_opt(card_opt, &e.time_range.to_string()),
                        ],
                    ));
                }
            }

            rows.sort_by_key(|(start, _)| *start);
            let rows: Vec<Vec<String>> = rows.into_iter().map(|(_, r)| r).collect();

            sections.push(ScheduleSection {
                title: format!("DATE: {}", date.format("%Y-%m-%d")),
                rows,
            });
        }

        sections
    }
}

fn paint_opt(card: Option<&Card>, s: &str) -> String {
    match card {
        Some(c) => c.color.paint(s),
        None => s.to_string(),
    }
}
