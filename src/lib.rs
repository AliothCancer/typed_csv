pub mod dataset_info;

pub mod struct_gen;
pub mod enum_gen;

pub mod sanitizer;

use std::io;

use csv::Reader;

use crate::{dataset_info::ColumnInfo, sanitizer::sanitize_identifier};


pub const COLUMN_TYPE_ENUM_NAME: &str= "CsvColumn";


#[derive(Debug)]
pub struct CsvDataset {
    pub names: Vec<ColName>,
    pub values: Vec<Vec<CsvAny>>,
    pub null_values: NullValues,
    pub info: Vec<ColumnInfo>
}

#[derive(Debug,Clone, Copy)]
pub struct ValueNamesView<'a>{
    values: &'a [Vec<CsvAny>],
    names: &'a [ColName]
}


#[derive(Debug, Clone)]
pub struct ColName{
    pub raw: String,
    pub sanitized: SanitizedStr
}


#[derive(Debug, Clone)]
pub struct SanitizedStr(String);

#[derive(Debug)]
pub struct NullValues(pub &'static [&'static str]);

impl CsvDataset {
    pub fn new<R: io::Read>(mut reader: Reader<R>, null_values: NullValues) -> Self {
        let names: Vec<ColName> = reader.headers().unwrap().iter().map(|str|{
            let sanitized = SanitizedStr(sanitize_identifier(str));
            let raw = str.to_string();
            ColName { raw, sanitized }
        }).collect();
        let mut values: Vec<Vec<CsvAny>> = (0..names.len()).map(|_|Vec::new()).collect();
        reader.into_records().for_each(|x| {
            x.unwrap()
                .iter()
                .enumerate()
                .for_each(|(column_index, value)| {
                    let k = values.get_mut(column_index).unwrap();
                    k.push(RawCsvValue(value).as_csvany(&null_values));
                })
        });

        Self {
            names,
            values,
            null_values,
            info: Vec::new(),
        }
    }
    pub fn view_names_and_values(&self) -> ValueNamesView<'_>{
        ValueNamesView { values: &self.values, names: &self.names }
    }
    pub fn split_view_and_info(&mut self) -> (ValueNamesView<'_>, &mut Vec<ColumnInfo>) {
        (
            ValueNamesView { 
                values: &self.values, 
                names: &self.names 
            },
            &mut self.info
        )
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

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub enum CsvAny {
    Str(String),
    Int(i64),
    Float(f64),
    Null,  // to represent null values
    Empty, // if it is just empty
}

// I was thinking about this alternative but
// it make it more complex
// #[derive(Debug, PartialEq, PartialOrd, Clone)]
// pub enum CsvInt {
//     Int(i64),
//     Empty,
//     Null,
// }

// #[derive(Debug, PartialEq, Clone)]
// pub enum CsvColumn {
//     IntColumn(Vec<CsvInt>),
//     FloatColumn(Vec<CsvFloat>),
//     StringColumn(Vec<CsvString>),
//     AnyColumn(Vec<CsvAny>),
// }
// #[derive(Debug, PartialEq, PartialOrd, Clone)]
// pub enum CsvFloat {
//     Float(f64),
//     Empty,
//     Null,
// }

// #[derive(Debug, PartialEq, PartialOrd, Clone)]
// pub enum CsvString {
//     Str(String),
//     Empty,
//     Null,
// }

// use std::convert::TryFrom;

// impl TryFrom<CsvAny> for CsvInt {
//     type Error = String;

//     fn try_from(value: CsvAny) -> Result<Self, Self::Error> {
//         match value {
//             CsvAny::Int(i) => Ok(CsvInt::Int(i)),
//             CsvAny::Empty => Ok(CsvInt::Empty),
//             CsvAny::Null => Ok(CsvInt::Null),
//             _ => Err(format!(
//                 "Type Mismatch: Cannot convert {:?} to CsvInt",
//                 value
//             )),
//         }
//     }
// }

// impl TryFrom<CsvAny> for CsvFloat {
//     type Error = String;

//     fn try_from(value: CsvAny) -> Result<Self, Self::Error> {
//         match value {
//             CsvAny::Float(f) => Ok(CsvFloat::Float(f)),
//             CsvAny::Empty => Ok(CsvFloat::Empty),
//             CsvAny::Null => Ok(CsvFloat::Null),
//             _ => Err(format!(
//                 "Type Mismatch: Cannot convert {:?} to CsvFloat",
//                 value
//             )),
//         }
//     }
// }

// impl TryFrom<CsvAny> for CsvString {
//     type Error = String;

//     fn try_from(value: CsvAny) -> Result<Self, Self::Error> {
//         match value {
//             CsvAny::Str(s) => Ok(CsvString::Str(s)),
//             CsvAny::Empty => Ok(CsvString::Empty),
//             CsvAny::Null => Ok(CsvString::Null),
//             _ => Err(format!(
//                 "Type Mismatch: Cannot convert {:?} to CsvString",
//                 value
//             )),
//         }
//     }
// }
