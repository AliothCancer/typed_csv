
/// Represents any possible value in a CSV cell.
///
/// # Examples
/// Can use into to convert to the corresponding CsvAny variant
/// ```
/// use crate::CsvAny;
///
/// let val: CsvAny = 42.into();
/// 
/// assert_eq!(val, CsvAny::Int(42));
/// ```
#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub enum CsvAny {
    Str(String),
    Int(i64),
    Float(f64),
    Null,  // to represent null values
    Empty, // if it is just empty
}
impl From<&str> for CsvAny {
    fn from(val: &str) -> Self {
        CsvAny::Str(val.to_string())
    }
}

impl From<i64> for CsvAny {
    fn from(val: i64) -> Self {
        CsvAny::Int(val)
    }
}
impl From<f64> for CsvAny{
    fn from(value: f64) -> Self {
        CsvAny::Float(value)
    }
}

