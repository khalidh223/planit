use crate::config::Config;
use crate::core::models::{Card, Event, Task};
use crate::core::repository::Repository;

use crate::logging::Logger;
use crate::errors::Result;
use std::path::PathBuf;

#[derive(Debug)]
pub struct AppContext {
    pub config: Config,
    pub tasks: Repository<Task>,
    pub events: Repository<Event>,
    pub cards: Repository<Card>,
    pub logger: Logger,
    pub startup_displayed: bool,
    pub config_path: PathBuf,
    pub schedules_dir: PathBuf,
    pub logs_dir: PathBuf,
}

impl AppContext {
    pub fn new() -> Self {
        Self::new_with_paths(
            PathBuf::from("config.json"),
            PathBuf::from("schedules"),
            PathBuf::from("logs"),
        )
        .unwrap()
    }

    pub fn new_with_paths(
        config_path: PathBuf,
        schedules_dir: PathBuf,
        logs_dir: PathBuf,
    ) -> Result<Self> {
        let config = Config::load_from(&config_path)?;
        let tasks = Repository::<Task>::new();
        let events = Repository::<Event>::new();
        let cards = Repository::<Card>::new();

        let logger = Logger::new();
        logger.set_log_dir(&logs_dir);
        logger.set_file_logging_enabled(config.file_logging_enabled());

        Ok(Self {
            config,
            tasks,
            events,
            cards,
            logger,
            startup_displayed: false,
            config_path,
            schedules_dir,
            logs_dir,
        })
    }
}
