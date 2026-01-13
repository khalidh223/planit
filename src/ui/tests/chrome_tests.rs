use crate::ui::{chrome::UiChrome, width_util::WidthUtil};

#[test]
fn ui_chrome_formats_centered_prefix_exactly() {
    let chrome = UiChrome::new();
    let util = WidthUtil::default();
    let box_width = 20;

    let prefix_line = chrome.format_centered_prefix("> ", box_width);

    let left_pad = util.center_pad(box_width);
    assert_eq!(prefix_line, format!("{}> ", " ".repeat(left_pad)));
}

#[test]
fn ui_chrome_formats_centered_lines() {
    let chrome = UiChrome::new();
    let util = WidthUtil::default();
    let box_width = 20;
    let left_pad = util.center_pad(box_width);

    let centered = chrome.format_centered_line("Hi", box_width);

    let inner_pad = (box_width - "Hi".len()) / 2;
    let expected = format!("{}{}Hi", " ".repeat(left_pad), " ".repeat(inner_pad));
    assert_eq!(centered, expected);
}
