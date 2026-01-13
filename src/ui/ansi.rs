// Shared ANSI/VT100 control sequences used across the UI.

/// Switch to the terminal's alternate screen buffer (smcup).
pub const ENTER_ALT_SCREEN: &str = crate::csi!("?1049h");
/// Return to the main screen buffer (rmcup).
pub const EXIT_ALT_SCREEN: &str = crate::csi!("?1049l");

/// Clear the entire screen.
pub const CLEAR_SCREEN: &str = crate::csi!("2J");
/// Move the cursor to the top-left corner.
pub const CURSOR_HOME: &str = crate::csi!("H");
/// Clear from cursor to end of line.
pub const CLEAR_LINE_REST: &str = crate::csi!("0K");
/// Move the cursor up one line.
pub const CURSOR_UP_ONE: &str = crate::csi!("1A");

/// Hide the cursor.
pub const HIDE_CURSOR: &str = crate::csi!("?25l");
/// Show the cursor.
pub const SHOW_CURSOR: &str = crate::csi!("?25h");
/// Request a blinking block cursor (if the terminal supports it).
pub const CURSOR_BLINKING_BLOCK: &str = crate::csi!("1 q");

/// Reset terminal styling to defaults.
pub const STYLE_RESET: &str = crate::csi!("0m");
/// Bold text.
pub const STYLE_BOLD: &str = crate::csi!("1m");
/// Italic text.
pub const STYLE_ITALIC: &str = crate::csi!("3m");
/// Light gray foreground.
pub const FG_LIGHT_GRAY: &str = crate::csi!("37m");
/// Dark gray background with white text for input prompts.
pub const PROMPT_STYLE: &str = crate::csi2!("38;5;15m", "48;5;236m");
