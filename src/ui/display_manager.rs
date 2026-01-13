use crate::config::Config;
use crate::core::models::{Card, Event, Task};
use crate::core::repository::Repository;
use crate::core::types::EntityType;
use crate::ui::display_data::{DisplayDataBuilder, ScheduleSection};
use crate::ui::table_printer::TablePrinter;
use crate::ui::width_util::WidthUtil;
use chrono::NaiveDate;
use std::io;
use std::io::Write;

#[derive(Debug, Default, Clone)]
pub struct DisplayManager {
    pub printer: TablePrinter,
    pub util: WidthUtil,
    pub data: DisplayDataBuilder,
}

impl DisplayManager {
    pub fn new() -> Self {
        Self {
            printer: TablePrinter::new(),
            util: WidthUtil::default(),
            data: DisplayDataBuilder::new(),
        }
    }

    pub fn display_config_centered(&self, config: &Config) -> usize {
        let headers = ["ID", "KEY", "DESCRIPTION", "VALUE"];
        let rows: Vec<Vec<String>> = config
            .rows()
            .iter()
            .enumerate()
            .map(|(i, (k, d, v))| vec![i.to_string(), k.clone(), d.clone(), v.clone()])
            .collect();

        let table_w = self
            .printer
            .compute_table_width(&headers, &rows)
            .max(self.util.visible_width("CONFIG"));

        let pad = self.util.center_pad(table_w);
        let printer = self.printer.with_left_pad(pad);

        printer.print_table(
            "Config",
            &headers,
            &rows,
            Some("No config items found."),
            Some(table_w),
        );
        table_w
    }

    pub fn display_config(&self, config: &Config) {
        let headers = ["ID", "KEY", "DESCRIPTION", "VALUE"];
        let rows: Vec<Vec<String>> = config
            .rows()
            .iter()
            .enumerate()
            .map(|(i, (k, d, v))| vec![i.to_string(), k.clone(), d.clone(), v.clone()])
            .collect();

        self.printer.print_table(
            "Config",
            &headers,
            &rows,
            Some("No config items found."),
            None,
        );
    }

    pub fn display_tasks(&self, tasks: &Repository<Task>, cards: &Repository<Card>) {
        let headers = ["ID", "NAME", "TAG", "HOURS", "DUE"];

        let rows = self.data.task_rows(tasks, cards);

        self.printer
            .print_table("Tasks", &headers, &rows, Some("No tasks available."), None);
    }

    pub fn display_events(&self, events: &Repository<Event>, cards: &Repository<Card>) {
        let headers = ["ID", "NAME", "TAG", "TIME", "DAYS", "RECURRING"];

        let rows = self.data.event_rows(events, cards);

        self.printer.print_table(
            "Events",
            &headers,
            &rows,
            Some("No events available."),
            None,
        );
    }

    pub fn display_cards(&self, cards: &Repository<Card>) {
        let headers = ["ID", "NAME", "COLOR"];
        let rows = self.data.card_rows(cards);

        self.printer
            .print_table("Cards", &headers, &rows, Some("No cards available."), None);
    }

    pub fn display_entities_for(
        &self,
        which: EntityType,
        tasks: &Repository<Task>,
        events: &Repository<Event>,
        cards: &Repository<Card>,
    ) {
        match which {
            EntityType::Task => self.display_tasks(tasks, cards),
            EntityType::Event => self.display_events(events, cards),
            EntityType::Card => self.display_cards(cards),
        }
    }

    pub fn render_schedule_for_days<W: Write>(
        &self,
        dates: &[NaiveDate],
        tasks: &Repository<Task>,
        events: &Repository<Event>,
        cards: &Repository<Card>,
        out: &mut W,
    ) -> io::Result<()> {
        let headers = ["ID", "NAME", "TAG", "HOURS", "TIME"];
        let empty_msg = "No tasks or events scheduled.";

        let sections = self
            .data
            .build_schedule_sections(dates, tasks, events, cards);
        let max_width = self.schedule_max_width(&sections, &headers, empty_msg);

        // banner
        self.printer.render_banner("Schedule", max_width, out)?;

        for s in &sections {
            let empty = if s.rows.is_empty() {
                Some(empty_msg)
            } else {
                None
            };
            self.printer
                .render_table(&s.title, &headers, &s.rows, empty, Some(max_width), out)?;
        }

        Ok(())
    }

    pub fn display_schedule_for_days(
        &self,
        dates: &[NaiveDate],
        tasks: &Repository<Task>,
        events: &Repository<Event>,
        cards: &Repository<Card>,
    ) {
        let mut stdout = io::stdout();
        let _ = self.render_schedule_for_days(dates, tasks, events, cards, &mut stdout);
    }

    fn schedule_max_width(
        &self,
        sections: &[ScheduleSection],
        headers: &[&str],
        empty_msg: &str,
    ) -> usize {
        let mut max_width = self.util.visible_width("SCHEDULE");
        for s in sections {
            let table_w = self.printer.compute_table_width(headers, &s.rows);
            let title_w = self.util.visible_width(&s.title);
            let empty_w = if s.rows.is_empty() {
                self.util.visible_width(empty_msg)
            } else {
                0
            };
            max_width = max_width.max(table_w.max(title_w).max(empty_w));
        }
        max_width
    }
}
