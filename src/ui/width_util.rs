use terminal_size::{Width, terminal_size};

use crate::ui::ascii::ESC_BYTE;
type ByteIter<'a> = std::iter::Peekable<std::str::Bytes<'a>>;

#[derive(Debug, Default, Clone)]
pub struct WidthUtil;

impl WidthUtil {
    fn strip_ansi(s: &str) -> String {
        let mut out = String::with_capacity(s.len());
        let mut bytes = s.bytes().peekable();

        while let Some(byte) = bytes.next() {
            if Self::is_escape_byte(byte) && Self::is_csi_start(bytes.peek()) {
                Self::consume_csi(&mut bytes);
                continue;
            }
            out.push(byte as char);
        }
        out
    }

    fn is_escape_byte(b: u8) -> bool {
        b == ESC_BYTE
    }

    fn is_csi_start(next: Option<&u8>) -> bool {
        matches!(next, Some(b'['))
    }

    fn consume_csi(bytes: &mut ByteIter<'_>) {
        let _ = bytes.next(); // skip '['
        while let Some(nb) = bytes.next() {
            if Self::is_csi_terminator(nb) {
                break;
            }
        }
    }

    fn is_csi_terminator(b: u8) -> bool {
        (b as char).is_ascii_alphabetic()
    }

    pub fn visible_width(&self, s: &str) -> usize {
        Self::strip_ansi(s).chars().count()
    }

    #[cfg(test)]
    pub(crate) fn strip_ansi_for_test(s: &str) -> String {
        Self::strip_ansi(s)
    }

    pub fn pad_visible(&self, s: &str, width: usize) -> String {
        let w = self.visible_width(s);
        if w >= width {
            s.to_string()
        } else {
            let mut out = String::with_capacity(s.len() + (width - w));
            out.push_str(s);
            for _ in 0..(width - w) {
                out.push(' ');
            }
            out
        }
    }

    /// Best-effort terminal width (defaults to 80).
    pub fn terminal_width(&self) -> usize {
        if let Some((Width(w), _)) = terminal_size() {
            w as usize
        } else {
            80
        }
    }

    /// Left padding to center a box of `content_width` inside the terminal.
    pub fn center_pad(&self, content_width: usize) -> usize {
        let tw = self.terminal_width();
        tw.saturating_sub(content_width) / 2
    }
}
