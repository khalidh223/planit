mod calendar_tests;
mod comparator_tests;
mod manager_tests;
mod overflow_tests;
mod packer_tests;

use crate::config::Config;
use crate::core::models::{Card, Event, Task};
use crate::core::{context::AppContext, repository::Repository};
use crate::logging::Logger;
use std::fs;
use std::path::PathBuf;

fn temp_config_path() -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("planit-scheduler-{nanos}.json"))
}

fn write_sample_config(path: &PathBuf) {
    let json = r#"
    {
      "range": { "value": "8:00AM-6:00PM", "description": "Daily hours" },
      "task_overflow_policy": { "value": "allow", "description": "overflow" },
      "task_scheduling_order": { "value": "longest-task-first", "description": "order" },
      "schedule_start_date": { "value": "2099-01-01", "description": "start date" },
      "file_logging_enabled": { "value": "True", "description": "file logging" }
    }
    "#;
    fs::write(path, json).unwrap();
}

pub(super) fn make_ctx() -> AppContext {
    let path = temp_config_path();
    write_sample_config(&path);
    let config = Config::load_from(&path).unwrap();
    let logger = Logger::new();
    let schedules_dir = std::env::temp_dir().join("planit-scheduler-schedules");
    let logs_dir = std::env::temp_dir().join("planit-scheduler-logs");
    logger.set_log_dir(&logs_dir);
    logger.set_file_logging_enabled(config.file_logging_enabled());
    AppContext {
        config,
        tasks: Repository::<Task>::new(),
        events: Repository::<Event>::new(),
        cards: Repository::<Card>::new(),
        logger,
        startup_displayed: false,
        config_path: path,
        schedules_dir,
        logs_dir,
    }
}

pub(super) fn sample_date() -> chrono::NaiveDate {
    chrono::NaiveDate::from_ymd_opt(2099, 1, 1).unwrap()
}
