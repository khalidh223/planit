// ASCII control codes and helpers for ANSI sequence composition.

/// ESC (escape) control character.
pub const ESC: &str = "\x1B";
/// ESC (escape) as a byte value.
pub const ESC_BYTE: u8 = 0x1B;

#[macro_export]
macro_rules! csi {
    ($suffix:literal) => {
        concat!("\x1B[", $suffix)
    };
}

#[macro_export]
macro_rules! csi2 {
    ($first:literal, $second:literal) => {
        concat!("\x1B[", $first, "\x1B[", $second)
    };
}
