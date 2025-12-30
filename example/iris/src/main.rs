#![allow(unused_variables)]

mod iris_latest_for_diff;
use std::{error::Error, fs::File};

use csv_deserializer::{csv_dataset::CsvDataset, NullValues};

//use crate::iris::*;
use crate::iris_latest_for_diff::*;

fn main() -> Result<(), Box<dyn Error>> {
    // Build a reader for the csv file
    let path = "iris.csv";
    let file = File::open(path)?;
    let rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(file);

    // builf the CsvDataset with reader and nullvalues
    let dataset = CsvDataset::new(rdr, NullValues(vec!["NA"]));

    // The iris.rs file is generate with the binary of csv_deserializer

    // Then inside the iris.rs file a CsvDataFrame is used
    // as the main struct which contains all the data
    let df = CsvDataFrame::new(&dataset);

    // Do ETL stuffes in a type safe way but it comes at less
    // flexibility sometimes, so you can always use CsvDataset which
    // use CsvAny as the type for every cell

    // Can destruct the column wrapper called CsvColumn with if let
    if let CsvColumn::target(target) = &df.target
        && let CsvColumn::petal_length_cm(_pet_length) = &df.petal_length_cm
    {
        target.iter().for_each(|x| match x {
            target::Iris_setosa => todo!(),
            target::Iris_versicolor => todo!(),
            target::Iris_virginica => todo!(),
            target::Null => todo!(),
        });
    }

    // Can use a list of all columns
    // make sure to use auto-completion
    // for match arms
    for col in df.get_columns() {
        match col {
            CsvColumn::sepal_length_cm(sepal_length_cms) => {
                sepal_length_cms.iter().for_each(|x|{
                    match x {
                        sepal_length_cm::Float(f) => todo!(),
                        sepal_length_cm::Null => todo!(),
                    }
                })
            },
            CsvColumn::sepal_width_cm(sepal_width_cms) => todo!(),
            CsvColumn::petal_length_cm(petal_length_cms) => todo!(),
            CsvColumn::petal_width_cm(petal_width_cms) => todo!(),
            CsvColumn::target(targets) => {
                targets.iter().for_each(|x|match x{
                    target::Iris_setosa => todo!(),
                    target::Iris_versicolor => todo!(),
                    target::Iris_virginica => todo!(),
                    target::Null => todo!(),
                })
            },
        }
    }

    Ok(())
}
