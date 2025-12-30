#![allow(unused, non_snake_case, non_camel_case_types)]
use csv_deserializer::{create_enum, csv_dataset::CsvDataset, csv_types::CsvAny};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum sepal_length_cm {
    Float(f64),
    Null,
}

impl std::str::FromStr for sepal_length_cm {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let f = s.parse::<f64>().unwrap();
        Ok(sepal_length_cm::Float(f))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum sepal_width_cm {
    Float(f64),
    Null,
}

impl std::str::FromStr for sepal_width_cm {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let f = s.parse::<f64>().unwrap();
        Ok(sepal_width_cm::Float(f))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum petal_length_cm {
    Float(f64),
    Null,
}

impl std::str::FromStr for petal_length_cm {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let f = s.parse::<f64>().unwrap();
        Ok(petal_length_cm::Float(f))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum petal_width_cm {
    Float(f64),
    Null,
}

impl std::str::FromStr for petal_width_cm {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let f = s.parse::<f64>().unwrap();
        Ok(petal_width_cm::Float(f))
    }
}

create_enum!(target;
"Iris-setosa" => Iris_setosa,
"Iris-versicolor" => Iris_versicolor,
"Iris-virginica" => Iris_virginica,
Null,
);

#[derive(Debug)]
pub enum CsvColumn {
    sepal_length_cm(Vec<sepal_length_cm>),
    sepal_width_cm(Vec<sepal_width_cm>),
    petal_length_cm(Vec<petal_length_cm>),
    petal_width_cm(Vec<petal_width_cm>),
    target(Vec<target>),
}

pub struct CsvDataFrame {
    pub sepal_length_cm: CsvColumn,
    pub sepal_width_cm: CsvColumn,
    pub petal_length_cm: CsvColumn,
    pub petal_width_cm: CsvColumn,
    pub target: CsvColumn,
}
impl CsvDataFrame {
    pub fn new(dataset: &CsvDataset) -> Self {
        let (index, _) = dataset
            .names
            .iter()
            .enumerate()
            .find(|(index, cl)| &cl.sanitized.0 == "sepal_length_cm")
            .unwrap();
        let sepal_length_cm = CsvColumn::sepal_length_cm(
            dataset.values[index]
                .iter()
                .map(|val| match val {
                    CsvAny::Float(f) => sepal_length_cm::Float(*f),
                    CsvAny::Null => sepal_length_cm::Null,

                    _ => panic!(),
                })
                .collect::<Vec<sepal_length_cm>>(),
        );

        let (index, _) = dataset
            .names
            .iter()
            .enumerate()
            .find(|(index, cl)| &cl.sanitized.0 == "sepal_width_cm")
            .unwrap();
        let sepal_width_cm = CsvColumn::sepal_width_cm(
            dataset.values[index]
                .iter()
                .map(|val| match val {
                    CsvAny::Float(f) => sepal_width_cm::Float(*f),
                    CsvAny::Null => sepal_width_cm::Null,

                    _ => panic!(),
                })
                .collect::<Vec<sepal_width_cm>>(),
        );

        let (index, _) = dataset
            .names
            .iter()
            .enumerate()
            .find(|(index, cl)| &cl.sanitized.0 == "petal_length_cm")
            .unwrap();
        let petal_length_cm = CsvColumn::petal_length_cm(
            dataset.values[index]
                .iter()
                .map(|val| match val {
                    CsvAny::Float(f) => petal_length_cm::Float(*f),
                    CsvAny::Null => petal_length_cm::Null,

                    _ => panic!(),
                })
                .collect::<Vec<petal_length_cm>>(),
        );

        let (index, _) = dataset
            .names
            .iter()
            .enumerate()
            .find(|(index, cl)| &cl.sanitized.0 == "petal_width_cm")
            .unwrap();
        let petal_width_cm = CsvColumn::petal_width_cm(
            dataset.values[index]
                .iter()
                .map(|val| match val {
                    CsvAny::Float(f) => petal_width_cm::Float(*f),
                    CsvAny::Null => petal_width_cm::Null,

                    _ => panic!(),
                })
                .collect::<Vec<petal_width_cm>>(),
        );

        let (index, _) = dataset
            .names
            .iter()
            .enumerate()
            .find(|(index, cl)| &cl.sanitized.0 == "target")
            .unwrap();
        let target = CsvColumn::target(
            dataset.values[index]
                .iter()
                .map(|val| match val {
                    CsvAny::Str(s) => target::from_str(s).unwrap(),
                    CsvAny::Null => target::Null,

                    _ => panic!(),
                })
                .collect::<Vec<target>>(),
        );

        CsvDataFrame {
            sepal_length_cm,
            sepal_width_cm,
            petal_length_cm,
            petal_width_cm,
            target,
        }
    }
    pub fn get_columns(&self) -> [&CsvColumn; 5] {
        [
            &self.sepal_length_cm,
            &self.sepal_width_cm,
            &self.petal_length_cm,
            &self.petal_width_cm,
            &self.target,
        ]
    }
}
