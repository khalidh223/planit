use crate::errors::{Error, Result};
use crate::prompter::models::{Flow, FlowCtrl};
use crate::ui::ansi::{
    CURSOR_BLINKING_BLOCK, CURSOR_HOME, ENTER_ALT_SCREEN, EXIT_ALT_SCREEN, HIDE_CURSOR, SHOW_CURSOR,
};
use std::io::{self, BufRead, BufReader, Write};

#[derive(Debug, Default, Clone)]
pub struct Prompter;

struct AltScreenGuard;
impl AltScreenGuard {
    fn enter() -> Self {
        print!("{ENTER_ALT_SCREEN}{CURSOR_HOME}");
        let _ = io::stdout().flush();
        Self
    }
}
impl Drop for AltScreenGuard {
    fn drop(&mut self) {
        print!("{SHOW_CURSOR}{EXIT_ALT_SCREEN}");
        let _ = io::stdout().flush();
    }
}

impl Prompter {
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    fn hide_cursor() {
        print!("{HIDE_CURSOR}");
        let _ = io::stdout().flush();
    }

    #[inline]
    fn show_cursor_blinking() {
        print!("{SHOW_CURSOR}{CURSOR_BLINKING_BLOCK}");
        let _ = io::stdout().flush();
    }

    pub fn run<F: Flow>(&self, flow: F, use_alt_screen: bool) -> Result<()> {
        let stdin = io::stdin();
        let reader = BufReader::new(stdin);
        self.run_with_reader(flow, use_alt_screen, reader)
    }

    pub fn run_with_reader<F: Flow, R: BufRead>(
        &self,
        mut flow: F,
        use_alt_screen: bool,
        mut reader: R,
    ) -> Result<()> {
        let _alt = if use_alt_screen {
            Some(AltScreenGuard::enter())
        } else {
            None
        };

        loop {
            // Redraw
            Self::hide_cursor();
            flow.render()?;
            Self::show_cursor_blinking();

            // Read input
            let mut line = String::new();
            let n = reader.read_line(&mut line).map_err(Error::Io)?;
            if n == 0 {
                return Ok(());
            }
            let line = line.trim();

            // Global escape hatch: typing "exit" leaves the alt screen immediately.
            if line.eq_ignore_ascii_case("exit") {
                return Ok(());
            }

            // Let the flow handle it
            match flow.handle_input(line)? {
                FlowCtrl::Continue => continue,
                FlowCtrl::Finish | FlowCtrl::Abort => return Ok(()),
            }
        }
    }
}
