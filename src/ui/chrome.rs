use crate::ui::ansi::{
    CLEAR_LINE_REST, CLEAR_SCREEN, CURSOR_HOME, CURSOR_UP_ONE, FG_LIGHT_GRAY, PROMPT_STYLE,
    STYLE_BOLD, STYLE_ITALIC, STYLE_RESET,
};
use crate::ui::width_util::WidthUtil;
use std::io::{self, Write};

/// Screen-level helpers (banner, clearing, centering prompts).
#[derive(Debug, Default, Clone)]
pub struct UiChrome {
    util: WidthUtil,
}

impl UiChrome {
    pub fn new() -> Self {
        Self {
            util: WidthUtil::default(),
        }
    }

    /// Compute the line for the banner and print it.
    pub fn print_banner(&self) {
        const INNER_WIDTH: usize = 50;
        let version = env!("CARGO_PKG_VERSION");
        let title = format!(
            "{STYLE_BOLD}P L A N I T{STYLE_RESET} {FG_LIGHT_GRAY}(v{version}){STYLE_RESET}"
        );
        let subtitle = format!("{STYLE_ITALIC}Scheduling made simple{STYLE_RESET}");
        println!("╭{}╮", "─".repeat(INNER_WIDTH));
        println!("│{}│", " ".repeat(INNER_WIDTH));
        println!("│{}│", self.center_in_box(&title, INNER_WIDTH));
        println!("│{}│", self.center_in_box(&subtitle, INNER_WIDTH));
        println!("│{}│", " ".repeat(INNER_WIDTH));
        println!("╰{}╯", "─".repeat(INNER_WIDTH));
    }

    pub fn clear_screen(&self) {
        print!("{CLEAR_SCREEN}{CURSOR_HOME}");
        let _ = io::stdout().flush();
    }

    pub fn print_centered_prefix(&self, prefix: &str, box_width: usize) {
        let line = self.format_centered_prefix(prefix, box_width);
        self.print_prompt_line(&line);
    }

    pub fn print_centered_prefix_plain(&self, prefix: &str, box_width: usize) {
        let line = self.format_centered_prefix(prefix, box_width);
        self.print_plain_line(&line);
    }

    pub fn print_prompt(&self, prompt: &str) {
        self.print_prompt_line(prompt);
    }

    pub fn print_prompt_plain(&self, prompt: &str) {
        self.print_plain_line(prompt);
    }

    pub fn println_centered_in_box(&self, s: &str, box_width: usize) {
        let line = self.format_centered_line(s, box_width);
        println!("{line}");
    }

    pub fn format_centered_prefix(&self, prefix: &str, box_width: usize) -> String {
        let left = self.util.center_pad(box_width);
        format!("{}{}", " ".repeat(left), prefix)
    }

    pub fn format_centered_line(&self, s: &str, box_width: usize) -> String {
        let left = self.util.center_pad(box_width);
        let inner_pad = if box_width <= self.util.visible_width(s) {
            0
        } else {
            (box_width - self.util.visible_width(s)) / 2
        };
        format!("{}{}{}", " ".repeat(left), " ".repeat(inner_pad), s)
    }

    fn print_prompt_line(&self, line: &str) {
        const PROMPT_TOP_PADDING_LINES: usize = 1;
        for _ in 0..PROMPT_TOP_PADDING_LINES {
            self.print_prompt_padding_line();
        }
        print!("{PROMPT_STYLE}{line}{CLEAR_LINE_REST}{STYLE_RESET}\n");
        print!("{PROMPT_STYLE}{CLEAR_LINE_REST}{STYLE_RESET}");
        let line_width = self.util.visible_width(line);
        let column = line_width + 1;
        print!("{CURSOR_UP_ONE}\x1B[{column}G{PROMPT_STYLE}");
        let _ = io::stdout().flush();
    }

    fn print_plain_line(&self, line: &str) {
        print!("{line}");
        let _ = io::stdout().flush();
    }

    pub fn print_prompt_bottom_padding(&self) {
        self.print_prompt_padding_line();
        let _ = io::stdout().flush();
    }

    fn print_prompt_padding_line(&self) {
        print!("{PROMPT_STYLE}{CLEAR_LINE_REST}{STYLE_RESET}\n");
    }

    fn center_in_box(&self, content: &str, width: usize) -> String {
        let content_width = self.util.visible_width(content);
        if content_width >= width {
            return content.to_string();
        }
        let left = (width - content_width) / 2;
        let right = width - content_width - left;
        format!("{}{}{}", " ".repeat(left), content, " ".repeat(right))
    }
}
