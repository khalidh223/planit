use crate::config::{ConfigKey, ConfigRows};
use crate::core::context::AppContext;
use crate::core::types::{TaskOverflowPolicy, TaskSchedulingOrder};
use crate::errors::Result;
use crate::prompter::models::{ConfigState, Flow, FlowCtrl};
use crate::logging::LogTarget;
use crate::ui::ansi::STYLE_RESET;
use crate::ui::chrome::UiChrome;
use crate::ui::display_manager::DisplayManager;
use std::io::Write;
use std::str::FromStr;
use strum::IntoEnumIterator;

pub struct ConfigEditFlow<'a> {
    ctx: &'a mut AppContext,
    dm: DisplayManager,
    chrome: UiChrome,
    state: ConfigState,
    rows_cache: ConfigRows,
    selected_idx: Option<usize>,
    pending_value: Option<String>,
    frame_width: usize,
}

impl<'a> ConfigEditFlow<'a> {
    pub fn new(ctx: &'a mut AppContext) -> Self {
        let rows = ctx.config.rows();
        Self {
            ctx,
            dm: DisplayManager::new(),
            chrome: UiChrome::new(),
            state: ConfigState::ShowTable,
            rows_cache: rows,
            selected_idx: None,
            pending_value: None,
            frame_width: 60,
        }
    }

    #[cfg(test)]
    pub(crate) fn state(&self) -> ConfigState {
        self.state.clone()
    }

    #[cfg(test)]
    pub(crate) fn selected_index(&self) -> Option<usize> {
        self.selected_idx
    }

    fn rows_len(&self) -> usize {
        self.rows_cache.len()
    }

    fn selected_row(&self) -> Option<(&str, &str, &str)> {
        self.selected_idx
            .and_then(|i| self.rows_cache.get(i))
            .map(|(k, d, v)| (k.as_str(), d.as_str(), v.as_str()))
    }

    fn refresh_rows(&mut self) {
        self.rows_cache = self.ctx.config.rows();
    }

    fn possible_options(&self) -> Option<String> {
        let (key, _, _) = self.selected_row()?;
        let parsed = ConfigKey::from_str(key).ok()?;
        match parsed {
            ConfigKey::TaskOverflowPolicy => Some(
                TaskOverflowPolicy::iter()
                    .map(|p| format!("{}: {}", p, p.help()))
                    .collect::<Vec<_>>()
                    .join("\n"),
            ),
            ConfigKey::TaskSchedulingOrder => Some(
                TaskSchedulingOrder::iter()
                    .map(|p| format!("{}: {}", p, p.help()))
                    .collect::<Vec<_>>()
                    .join("\n"),
            ),
            ConfigKey::FileLoggingEnabled => Some(
                vec![
                    "True: enable writing log messages to the session file",
                    "False: disable writing log messages to the session file",
                ]
                .join("\n"),
            ),
            ConfigKey::Range => None,
            ConfigKey::ScheduleStartDate => None,
        }
    }

    fn render_prompt(&self, message: &str) {
        self.chrome
            .println_centered_in_box(message, self.frame_width);
        self.chrome
            .print_centered_prefix_plain("> ", self.frame_width);
    }

    fn render_selected_details(&self) {
        if let Some((_k, desc, val)) = self.selected_row() {
            self.chrome.println_centered_in_box(desc, self.frame_width);
            self.chrome
                .println_centered_in_box(&format!("Current value: {val}"), self.frame_width);
        }
    }

    fn render_possible_options(&self) {
        if let Some(opts) = self.possible_options() {
            self.chrome.println_centered_in_box(
                &format!("Possible options: {opts}"),
                self.frame_width,
            );
        }
    }

    fn render_new_value_prompt(&mut self, show_details: bool) {
        if show_details {
            self.render_selected_details();
        }
        self.render_possible_options();
        self.render_prompt("Enter new value: ");
    }

    fn render_table(&mut self) {
        self.chrome.clear_screen();
        self.frame_width = self.dm.display_config_centered(&self.ctx.config);
        self.render_prompt("Would you like to edit a setting? (Y/N)");
    }

    fn render_select_id(&self) {
        self.render_prompt(&format!(
            "Enter ID (0..{}): ",
            self.rows_len().saturating_sub(1)
        ));
    }
}

impl<'a> Flow for ConfigEditFlow<'a> {
    fn render(&mut self) -> Result<()> {
        match self.state {
            ConfigState::ShowTable => self.render_table(),
            ConfigState::SelectId => self.render_select_id(),
            ConfigState::ShowCurrent => {
                self.render_new_value_prompt(true);
                self.state = ConfigState::AskNewValue;
            }
            ConfigState::AskNewValue => {
                self.render_new_value_prompt(false);
            }
            ConfigState::ApplyChange => { /* no-op */ }
            ConfigState::Done => { /* no-op */ }
        }
        Ok(())
    }

    fn handle_input(&mut self, input: &str) -> Result<FlowCtrl> {
        print!("{STYLE_RESET}");
        let _ = std::io::stdout().flush();
        match self.state {
            // Y/N at table
            ConfigState::ShowTable => self.handle_table_input(input),

            ConfigState::SelectId => self.handle_select_id_input(input),

            // auto-advanced by render()
            ConfigState::ShowCurrent => Ok(FlowCtrl::Continue),

            ConfigState::AskNewValue => self.handle_new_value_input(input),

            ConfigState::ApplyChange => Ok(FlowCtrl::Continue),

            ConfigState::Done => Ok(FlowCtrl::Finish),
        }
    }
}

impl<'a> ConfigEditFlow<'a> {
    fn handle_table_input(&mut self, input: &str) -> Result<FlowCtrl> {
        match input {
            "y" | "Y" => {
                self.state = ConfigState::SelectId;
            }
            "n" | "N" => {
                self.state = ConfigState::Done;
                return Ok(FlowCtrl::Finish);
            }
            _ => {
                self.chrome
                    .println_centered_in_box("Please enter Y or N.", self.frame_width);
            }
        }
        Ok(FlowCtrl::Continue)
    }

    fn handle_select_id_input(&mut self, input: &str) -> Result<FlowCtrl> {
        let len = self.rows_len();
        if len == 0 {
            self.chrome
                .println_centered_in_box("No config items to edit.", self.frame_width);
            self.state = ConfigState::Done;
            return Ok(FlowCtrl::Finish);
        }

        match input.parse::<usize>() {
            Ok(v) if v < len => {
                self.selected_idx = Some(v);
                self.state = ConfigState::ShowCurrent;
            }
            _ => {
                self.chrome.println_centered_in_box(
                    &format!("Invalid ID. Please enter 0..{}.", len.saturating_sub(1)),
                    self.frame_width,
                );
            }
        }
        Ok(FlowCtrl::Continue)
    }

    fn handle_new_value_input(&mut self, input: &str) -> Result<FlowCtrl> {
        self.pending_value = Some(input.to_string());
        self.state = ConfigState::ApplyChange;

        if let Some(idx) = self.selected_idx {
            if let Some(new_val) = self.pending_value.clone() {
                self.apply_config_change(idx, &new_val)?;
            }
        }

        Ok(FlowCtrl::Continue)
    }

    fn apply_config_change(&mut self, idx: usize, new_val: &str) -> Result<()> {
        match self.ctx.config.set_by_index(idx, new_val) {
            Ok(()) => {
                if let Some((key, _, _)) = self.rows_cache.get(idx) {
                    self.chrome
                        .println_centered_in_box(&format!("Updated {key}."), self.frame_width);
                }

                if let Some((key, old, new)) = self.ctx.config.take_last_change() {
                    self.log_config_change(key, old, new);
                }

                self.ctx
                    .logger
                    .set_file_logging_enabled(self.ctx.config.file_logging_enabled());

                self.refresh_rows();
                self.state = ConfigState::ShowTable;
            }
            Err(e) => {
                self.chrome
                    .println_centered_in_box(&format!("Error: {e}"), self.frame_width);
                self.state = ConfigState::AskNewValue;
            }
        }

        Ok(())
    }

    fn log_config_change(&mut self, key: String, old: String, new: String) {
        let file_enabled = self.ctx.config.file_logging_enabled();
        let is_file_logging_key = matches!(
            ConfigKey::from_str(&key),
            Ok(ConfigKey::FileLoggingEnabled)
        );
        if is_file_logging_key {
            if file_enabled {
                self.ctx.logger.set_file_logging_enabled(true);
            }
            self.ctx.logger.info(
                format!("Config '{}' updated: '{}' -> '{}'", key, old, new),
                LogTarget::FileOnly,
            );
            if !file_enabled {
                self.ctx.logger.set_file_logging_enabled(false);
            }
        } else {
            self.ctx.logger.info(
                format!("Config '{}' updated: '{}' -> '{}'", key, old, new),
                LogTarget::FileOnly,
            );
        }
    }
}
