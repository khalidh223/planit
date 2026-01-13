use strum::IntoEnumIterator;

trait EnumValidCsv: IntoEnumIterator + AsRef<str> + Sized {
    fn valid_csv() -> String {
        Self::iter()
            .map(|v| v.as_ref().to_owned())
            .collect::<Vec<_>>()
            .join(", ")
    }
}
impl<T> EnumValidCsv for T where T: IntoEnumIterator + AsRef<str> + Sized {}
pub fn valid_csv<T>() -> String
where
    T: IntoEnumIterator + AsRef<str> + Sized,
{
    <T as EnumValidCsv>::valid_csv()
}
