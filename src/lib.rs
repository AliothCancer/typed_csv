pub mod csv_dataset;
pub mod csv_types;
pub mod dataset_info;
pub mod enum_gen;
pub mod sanitizer;
pub mod struct_gen;

use crate::{csv_types::CsvAny, sanitizer::sanitize_identifier};

pub const COLUMN_TYPE_ENUM_NAME: &str = "CsvColumn";
pub const MAIN_STRUCT_NAME: &str = "CsvDataFrame";

/// A view over all the column names and
/// all the values
#[derive(Debug, Clone, Copy)]
pub struct ValueNamesView<'a> {
    pub values: &'a [Vec<CsvAny>],
    pub names: &'a [ColName],
}

/// The mutable version of the view over all the column names and
/// all the values
#[derive(Debug)]
pub struct ValueNamesMut<'a> {
    pub values: &'a [Vec<CsvAny>],
    pub names: &'a [ColName],
}

#[derive(Debug)]
pub struct RemovedColumn{
    pub col_values: Vec<CsvAny>,
    pub name: ColName,
}

#[derive(Debug, Clone)]
pub struct ColName {
    pub raw: String,
    pub sanitized: SanitizedStr,
}

#[derive(Debug, Clone)]
pub struct SanitizedStr(pub String);

#[derive(Debug, Default)]
pub struct NullValues<'a>(pub Vec<&'a str>);

impl ColName {
    pub fn new(raw: &str) -> Self {
        let sanitized = SanitizedStr(sanitize_identifier(raw));
        Self {
            raw: raw.to_string(),
            sanitized,
        }
    }
}

#[derive(Debug)]
struct RawCsvValue<'reader>(&'reader str);

impl<'reader> RawCsvValue<'reader> {
    fn as_csvany(&self, null_values: &NullValues) -> CsvAny {
        if self.0.is_empty() {
            return CsvAny::Empty;
        } else if null_values.0.contains(&self.0) {
            /*  */
            return CsvAny::Null;
        }

        let try_float = self.0.parse::<f64>();
        let try_int = self.0.parse::<i64>();

        match (try_float, try_int) {
            (_, Ok(int)) => CsvAny::Int(int),
            (Ok(float), Err(_)) => CsvAny::Float(float),
            (Err(_), Err(_)) => CsvAny::Str(self.0.to_owned()),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::csv_dataset::CsvDataset;

    use super::*;

    #[test]
    fn populate_info() {
        // 1. Setup Column Names
        let names = vec![ColName::new("mixed_data"), ColName::new("another")];
        let col1 = vec![
            10.into(),
            10.into(),
            20.into(),
            (21.4).into(),
            (21.4).into(),
            "Hello".into(),
            "Hello".into(),
            "hello".into(),
            CsvAny::Null,
            CsvAny::Null,
            CsvAny::Empty,
            CsvAny::Empty,
        ];
        let col2 = vec![
            101.into(),
            (-101).into(),
            20.into(),
            (21.4).into(),
            (21.4).into(),
            "Hello World ".into(),
            "Hello Worlds".into(),
            "hello wolrs".into(),
            CsvAny::Null,
            CsvAny::Null,
            CsvAny::Empty,
            CsvAny::Empty,
        ];

        let values = vec![col1, col2];

        let mut df = CsvDataset {
            names,
            values,
            null_values: NullValues(Vec::new()),
            info: Vec::new(),
        };

        CsvDataset::populate_column_infos(&mut df);

        // Ensure info was created for each column
        assert_eq!(df.info.len(), 2);

        // check the first column only
        let info1 = &df.info[0];

        // --- Assert Counters (Total occurrences, NOT unique) ---
        // We had 3 Ints total (10, 10, 20)
        assert_eq!(info1.number_of_ints, 3, "Should count 3 integers total");

        assert_eq!(info1.number_of_strings, 3, "Should count 3 string");

        assert_eq!(info1.number_of_nulls, 2, "Should count 2 null");

        assert_eq!(info1.number_of_empties, 2, "Should count 2 empty");

        assert_eq!(info1.number_of_floats, 2, "Should count 2 floats");

        assert_eq!(info1.unique_values.len(), 7, "Should have 7 unique values");

        // Verify sanitized name on the Info struct matches input
        assert_eq!(info1.column_name.raw, "mixed_data");
    }
}
