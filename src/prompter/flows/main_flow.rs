use std::io::{self, Write};

use crate::arg::arg_parse_strategy::CommandArgParser;
use crate::arg::args::Arg;
use crate::command::command_parser::CommandParser;
use crate::command::commands::CommandDyn;
use crate::command::manual::ManualCatalog;
use crate::core::context::AppContext;
use crate::errors::Result;
use crate::logging::{LogTarget, Logger};
use crate::prompter::models::{Flow, FlowCtrl};
use crate::ui::ansi::STYLE_RESET;
use crate::ui::chrome::UiChrome;

pub struct MainFlow<'a> {
    ctx: &'a mut AppContext,
    arg_parser: CommandArgParser,
    command_parser: CommandParser,
    logger: Logger,
}

impl<'a> MainFlow<'a> {
    pub fn new(ctx: &'a mut AppContext) -> Self {
        let logger = ctx.logger.clone();
        Self {
            ctx,
            arg_parser: CommandArgParser::new(),
            command_parser: CommandParser::new(),
            logger,
        }
    }
}

impl<'a> Flow for MainFlow<'a> {
    fn render(&mut self) -> Result<()> {
        self.print_startup();
        self.print_prompt();
        Ok(())
    }

    fn handle_input(&mut self, input: &str) -> Result<FlowCtrl> {
        self.prepare_output_space();
        let line = input.trim();
        if let Some(ctrl) = self.handle_non_command(line) {
            return Ok(ctrl);
        }

        let (raw_command, raw_args) = self.split_command_line(line);

        let args = match self.parse_args(&raw_command, &raw_args, line) {
            Some(args) => args,
            None => return Ok(FlowCtrl::Continue),
        };

        let cmd = match self.resolve_command(&raw_command, &args) {
            Some(cmd) => cmd,
            None => return Ok(FlowCtrl::Continue),
        };

        self.log_command_run(&raw_command, line);

        self.execute_command(&raw_command, cmd);

        Ok(FlowCtrl::Continue)
    }
}

impl<'a> MainFlow<'a> {
    fn print_startup(&mut self) {
        if self.ctx.startup_displayed {
            return;
        }
        let chrome = UiChrome::new();
        chrome.print_banner();
        println!();
        println!("Use 'man <topic>' for command-specific details.");
        let topics = ManualCatalog::new().topics();
        println!("Available topics: {}", topics.join(", "));
        println!();
        println!("Config path: {}", self.ctx.config_path.display());
        println!("Schedules path: {}", self.ctx.schedules_dir.display());
        println!("Logs path: {}", self.ctx.logs_dir.display());
        println!();
        self.ctx.startup_displayed = true;
    }

    fn print_prompt(&self) {
        UiChrome::new().print_prompt("> ");
    }

    fn prepare_output_space(&self) {
        UiChrome::new().print_prompt_bottom_padding();
        println!();
        print!("{STYLE_RESET}");
        let _ = io::stdout().flush();
    }

    fn handle_non_command(&self, line: &str) -> Option<FlowCtrl> {
        if line.is_empty() {
            return Some(FlowCtrl::Continue);
        }
        if line.eq_ignore_ascii_case("exit") {
            return Some(FlowCtrl::Finish);
        }
        None
    }

    fn split_command_line(&self, line: &str) -> (String, Vec<String>) {
        let mut raw_parts = line.split_whitespace();
        let raw_command = raw_parts.next().unwrap_or("").to_string();
        let raw_args = raw_parts.map(|s| s.to_string()).collect();
        (raw_command, raw_args)
    }

    fn parse_args(&self, raw_command: &str, raw_args: &[String], line: &str) -> Option<Vec<Arg>> {
        match self.arg_parser.parse(raw_command, raw_args) {
            Ok(args) => Some(args),
            Err(err) => {
                self.logger.error(
                    format!("Argument parsing failed for '{line}'. {err}"),
                    LogTarget::ConsoleAndFile,
                );
                None
            }
        }
    }

    fn resolve_command<'b>(&self, raw_command: &str, args: &'b [Arg]) -> Option<CommandDyn<'b>> {
        match self.command_parser.parse(raw_command, args) {
            Ok(cmd) => Some(cmd),
            Err(err) => {
                self.logger.error(
                    format!("Command resolution failed for '{raw_command}'. {err}"),
                    LogTarget::ConsoleAndFile,
                );
                None
            }
        }
    }

    fn log_command_run(&self, raw_command: &str, line: &str) {
        if !raw_command.eq_ignore_ascii_case("log") {
            self.logger
                .info(format!("Command run: {}", line), LogTarget::FileOnly);
        }
    }

    fn execute_command(&mut self, raw_command: &str, cmd: CommandDyn<'_>) {
        if let Err(err) = cmd.execute(self.ctx) {
            self.handle_command_error(raw_command, err.to_string());
        }
    }

    fn handle_command_error(&self, raw_command: &str, err_text: String) {
        if let Some(usage_error) = self.format_usage_error(raw_command, &err_text) {
            self.logger
                .error(usage_error.console_msg, LogTarget::ConsoleOnly);
            self.logger.error(usage_error.file_msg, LogTarget::FileOnly);
            return;
        }

        self.logger.error(
            format!("Command execution failed for '{raw_command}'. {err_text}"),
            LogTarget::ConsoleAndFile,
        );
    }

    fn format_usage_error(&self, raw_command: &str, err_text: &str) -> Option<UsageErrorMessage> {
        let (head, tail) = err_text.split_once("\nUsage:")?;
        let console_msg =
            format!("Command execution failed for '{raw_command}'. {head}\nUsage:{tail}");
        let file_msg = format!(
            "Command execution failed for '{raw_command}'. {}",
            head.trim()
        );
        Some(UsageErrorMessage {
            console_msg,
            file_msg,
        })
    }
}

struct UsageErrorMessage {
    console_msg: String,
    file_msg: String,
}
