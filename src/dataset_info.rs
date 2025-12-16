use std::fmt::Display;

use itertools::Itertools;

use crate::{ColName, CsvAny, ValueNamesView, sanitizer::sanitize_identifier};

#[derive(Debug, Clone)]
pub struct ColumnInfo {
    pub column_name: ColName,
    pub number_of_empties: u32,
    pub number_of_nulls: u32,
    pub number_of_strings: u32,
    pub number_of_floats: u32,
    pub number_of_ints: u32,
    pub unique_values: Vec<Variant>,
}

#[derive(Debug, Clone)]
pub struct Variant {
    pub raw: String,
    pub sanitized: String,
    pub csvany: CsvAny,
}

impl ColumnInfo {
    pub fn new(names_and_values_view: ValueNamesView, column_name: &str) -> Self {
        let ValueNamesView { values, names } = names_and_values_view;
        let (column_index, column_name) = names
            .iter()
            .enumerate()
            .find(|(_, x)| column_name.contains(x.raw.as_str()))
            .unwrap_or_else(|| panic!("No column named {column_name} found!"));

        let mut number_of_empties = 0;
        let mut number_of_nulls: u32 = 0;
        let mut number_of_strings: u32 = 0;
        let mut number_of_floats: u32 = 0;
        let mut number_of_ints: u32 = 0;

        let mut values: Vec<&CsvAny> = values[column_index].iter().collect();
        values.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let unique_values = values
            .into_iter()
            .inspect(|&x| {
                match x {
                    CsvAny::Str(_) => number_of_strings += 1,
                    CsvAny::Int(_) => number_of_ints += 1,
                    CsvAny::Float(_) => number_of_floats += 1,
                    CsvAny::Null => number_of_nulls += 1,
                    CsvAny::Empty => number_of_empties += 1,
                };
            })
            .dedup_by(|a, b| a == b)
            .cloned()
            .map(|unique_val| match unique_val {
                CsvAny::Str(str) => Variant {
                    raw: str.clone(),
                    sanitized: sanitize_identifier(&str),
                    csvany: CsvAny::Str(str),
                },
                CsvAny::Int(i) => {
                    let raw = i.to_string();
                    let sanitized = sanitize_identifier(&raw);
                    Variant {
                        raw,
                        sanitized,
                        csvany: CsvAny::Int(i),
                    }
                }
                CsvAny::Null => Variant {
                    raw: "Null".to_string(),
                    sanitized: "Null".to_string(),
                    csvany: CsvAny::Null,
                },
                CsvAny::Empty => Variant {
                    raw: "Empty".to_string(),
                    sanitized: "Empty".to_string(),
                    csvany: CsvAny::Empty,
                },
                CsvAny::Float(f) => Variant {
                    raw: f.to_string(),
                    sanitized: "".to_string(),
                    csvany: CsvAny::Float(f),
                },
            })
            .collect::<Vec<Variant>>();

        Self {
            column_name: column_name.clone(),
            number_of_empties,
            number_of_nulls,
            number_of_strings,
            number_of_floats,
            number_of_ints,
            unique_values,
        }
    }
}

impl Display for ColumnInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let render = [
            (self.number_of_empties, "Empties"),
            (self.number_of_nulls, "Nulls"),
            (self.number_of_strings, "Strings"),
            (self.number_of_floats, "Floats"),
            (self.number_of_ints, "Ints"),
        ]
        .into_iter()
        .map(|(x, str)| match x {
            0 => "".to_string(),
            n => format!("\n\t{str}: {n}"),
        })
        .collect::<String>();

        let unique_values = self
            .unique_values
            .iter()
            .map(|x| format!("\n\t{:?}", x))
            .collect::<String>();
        write!(
            f,
            "Name: {}\n\nTypes:{}\n\nUnique Values:{}",
            self.column_name.sanitized.0, render, unique_values
        )
    }
}
