use crate::core::types::CardColor;
use crate::ui::width_util::WidthUtil;

#[test]
fn width_util_strips_ansi_for_visible_width() {
    let util = WidthUtil::default();
    let s = format!("{}Red{}", CardColor::Red.ansi_fg(), CardColor::RESET);
    assert_eq!(util.visible_width(&s), 3);
}

#[test]
fn width_util_strip_ansi_removes_sequences() {
    let s = format!("{}Blue{}", CardColor::Blue.ansi_fg(), CardColor::RESET);
    assert_eq!(WidthUtil::strip_ansi_for_test(&s), "Blue");
}

#[test]
fn width_util_pad_visible_preserves_width() {
    let util = WidthUtil::default();
    let padded = util.pad_visible("abc", 5);
    assert_eq!(padded.len(), 5);
}

#[test]
fn width_util_center_pad_uses_terminal_width() {
    let util = WidthUtil::default();
    let pad = util.center_pad(10);
    assert!(pad <= util.terminal_width());
}
