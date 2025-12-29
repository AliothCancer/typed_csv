pub mod dataset_info;

pub mod enum_gen;
pub mod struct_gen;

pub mod csv_type;
pub mod sanitizer;

use std::io;

use csv::Reader;

use crate::{
    csv_type::CsvAny,
    dataset_info::{ColumnInfo, Variant},
    sanitizer::sanitize_identifier,
};

pub const COLUMN_TYPE_ENUM_NAME: &str = "CsvColumn";
pub const MAIN_STRUCT_NAME: &str = "CsvDataFrame";

/// This is a form to represent the dataset
/// which does not deep typization but can still
/// be usefull, also info field holds some info about the
/// types for each column
#[derive(Debug, Default)]
pub struct CsvDataset<'a> {
    pub names: Vec<ColName>,
    pub values: Vec<Vec<CsvAny>>,
    pub null_values: NullValues<'a>,
    pub info: Vec<ColumnInfo>,
}

#[derive(Debug, Clone, Copy)]
pub struct ValueNamesView<'a> {
    pub values: &'a [Vec<CsvAny>],
    pub names: &'a [ColName],
}
#[derive(Debug)]
pub struct ValueNamesMut<'a> {
    pub values: &'a [&'a mut Vec<CsvAny>],
    pub names: &'a [&'a mut ColName],
}

#[derive(Debug, Clone)]
pub struct ColName {
    pub raw: String,
    pub sanitized: SanitizedStr,
}

impl ColName {
    pub fn new(raw: &str) -> Self {
        let sanitized = SanitizedStr(sanitize_identifier(raw));
        Self {
            raw: raw.to_string(),
            sanitized,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SanitizedStr(pub String);

#[derive(Debug, Default)]
pub struct NullValues<'a>(pub Vec<&'a str>);

impl<'a> CsvDataset<'a> {
    /// Lenght of column values are not checked, so every column can have
    /// different lenght, be aware of row indexing
    pub fn push(&mut self, col_name: &str, col_values: Vec<CsvAny>) {
        self.names.push(ColName::new(col_name));
        self.values.push(col_values);
    }

    pub fn new<R: io::Read>(mut reader: Reader<R>, null_values: NullValues<'a>) -> Self {
        let names: Vec<ColName> = reader
            .headers()
            .unwrap()
            .iter()
            .map(|str| {
                let sanitized = SanitizedStr(sanitize_identifier(str));
                let raw = str.to_string();
                ColName { raw, sanitized }
            })
            .collect();
        let mut values: Vec<Vec<CsvAny>> = (0..names.len()).map(|_| Vec::new()).collect();
        reader.into_records().for_each(|x| {
            x.unwrap()
                .iter()
                .enumerate()
                .for_each(|(column_index, value)| {
                    let k = values.get_mut(column_index).unwrap();
                    k.push(RawCsvValue(value).as_csvany(&null_values));
                });
        });

        Self {
            names,
            values,
            null_values,
            info: Vec::new(),
        }
    }
    pub fn view_names_and_values(&self) -> ValueNamesView<'_> {
        ValueNamesView {
            values: &self.values,
            names: &self.names,
        }
    }
    pub fn split_view_and_info(&mut self) -> (ValueNamesView<'_>, &mut Vec<ColumnInfo>) {
        (
            ValueNamesView {
                values: &self.values,
                names: &self.names,
            },
            &mut self.info,
        )
    }

    /// Analyze every cell in the csv file to extract every unique value
    pub fn populate_column_infos(dataset: &mut Self) {
        let (value_names_view, info) = dataset.split_view_and_info();
        let col_name = value_names_view.names;

        for col_name in col_name {
            let mut col_info = ColumnInfo::new(value_names_view, &col_name.raw);

            if !col_info
                .unique_values
                .iter()
                .any(|x| x.csvany == CsvAny::Null)
            {
                let str = String::from("Null");
                col_info.unique_values.push(Variant {
                    raw: str.clone(),
                    sanitized: str,
                    csvany: CsvAny::Null,
                });
            }

            info.push(col_info.clone());
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
        // 2. Setup Values
        // We create mixed_data column with specific counts to test the counters:
        // - 2 Ints (Value: 10) -> Duplicate value to test deduplication
        // - 1 Int (Value: 20)
        // - 2 floats
        // - 3 String
        // - 2 Null
        // - 2 Empty
        let values = vec![col1, col2];

        let mut df = CsvDataset {
            names,
            values,
            null_values: NullValues(Vec::new()),
            info: Vec::new(),
        };

        // 3. Execute logic
        CsvDataset::populate_column_infos(&mut df);

        // 4. Assertions

        // Ensure info was created
        assert_eq!(df.info.len(), 2);
        let info1 = &df.info[0];

        // --- Assert Counters (Total occurrences, NOT unique) ---
        // We had 3 Ints total (10, 10, 20)
        assert_eq!(info1.number_of_ints, 3, "Should count 3 integers total");
        // We had 1 String
        assert_eq!(info1.number_of_strings, 3, "Should count 2 string");
        // We had 1 Null
        assert_eq!(info1.number_of_nulls, 2, "Should count 1 null");
        // We had 1 Empty
        assert_eq!(info1.number_of_empties, 2, "Should count 1 empty");
        // We had 0 Floats
        assert_eq!(info1.number_of_floats, 2, "Should count 0 floats");

        // --- Assert Unique Values (Deduplicated) ---
        // Expected uniques: Int(10), Int(20), Str("Hello"), Null, Empty
        // Total unique count should be 5
        assert_eq!(info1.unique_values.len(), 7, "Should have 7 unique values");

        // Verify sanitized name on the Info struct matches input
        assert_eq!(info1.column_name.raw, "mixed_data");
    }
}
