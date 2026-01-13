pub trait ToDashSeparators {
    /// Returns a copy with all `/` replaced by `-` and leading/trailing
    /// whitespace trimmed.
    fn to_dash_separators(&self) -> String;
}

impl ToDashSeparators for str {
    fn to_dash_separators(&self) -> String {
        self.trim().replace('/', "-")
    }
}

impl ToDashSeparators for String {
    fn to_dash_separators(&self) -> String {
        self.as_str().to_dash_separators()
    }
}
