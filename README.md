# What is this repo?
This repo contains a rust binary (main.rs) which translate a csv table in rust types, every column is converted into a Vec of enum representing all the unique types, if a column is of String type then every unique String will be deserialized as an enum variant (see iris dataset example).

- The binary will output all the rust code to stdout so it can be easily piped to write a file via terminal.

# Bin usage
```bash
❯ cargo run --release --quiet -- -h
Usage: csv_deserializer [OPTIONS] --input-file <input_file>

Options:
  -i, --input-file <input_file>
  -n, --null-values <a,b,..>
  -h, --help                     Print help
  -V, --version                  Print version
```

*Note on null values:*
- `--null-values` is an optional comma separate list of string which will be converted to the Null variant which all generated enums have

# Lib Usage Guide

To use this library for generating and utilizing a typed Rust interface for your CSV files, follow these steps:

## 1. Loading the Dataset
First, load your CSV file using a `csv::Reader`. You then create a `CsvDataset` by providing the reader and specifying which strings should be treated as null values.

```rust
let file = File::open("iris.csv")?;
let rdr = csv::ReaderBuilder::new()
    .has_headers(true)
    .from_reader(file);

let mut dataset = CsvDataset::new(rdr, NullValues(&["NA"]));
```

## 2. Generating Rust Code
Use the csv_deserializing cli to generate the rust code for a specific csv file. The binary will print all the rust code so you can redirect this output to a file from your command line to save it.

## 3. Using the Generated Code
Once the code is saved into a file (e.g., `iris.rs`), you can import it into your project. To work with the typed data, initialize a `CsvDataFrame` type by passing the `CsvDataset` you created earlier.

```rust
mod iris;
use iris::*;

let df = CsvDataFrame::new(dataset);
```

## 4. Iris Dataset ETL Example
```rust
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
    // make sure to use completion
    // for match arms
    for col in df.get_columns() {
        match col {
            CsvColumn::sepal_length_cm(sepal_length_cms) => todo!(),
            CsvColumn::sepal_width_cm(sepal_width_cms) => todo!(),
            CsvColumn::petal_length_cm(petal_length_cms) => todo!(),
            CsvColumn::petal_width_cm(petal_width_cms) => todo!(),
            CsvColumn::target(targets) => todo!(),
        }
    }

```
# More info
## Name sanitization and Type Recognition: Categorical vs Numerical
Sanitization is achived converting any number or special char to Strings that will be used in the generated code. In particular the function which does it is contained in sanitizer.rs (`sanitize_identifier`).

The library identifies types by attempting to parse each raw CSV value.
* **Numerical**: If a value parses as an `i64`, it is treated as an `Int`; if it parses as an `f64`, it is treated as a `Float`. For example taking a look at `sepal length (cm)` in the iris dataset, the resulting type is:
```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum sepal_length_cm {
    Float(f64),
    Null,
}
// Also implement from string
impl std::str::FromStr for sepal_length_cm {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let f = s.parse::<f64>().unwrap();
        Ok(sepal_length_cm::Float(f))
    }
}
```

* **Categorical**: Values that cannot be parsed as numbers are treated as `Str`. The generated rust code for a string values column is like: (Example for iris dataset)
```rust
create_enum!(target;
"Iris-setosa" => Iris_setosa,
"Iris-versicolor" => Iris_versicolor,
"Iris-virginica" => Iris_virginica,
Null,
);
```
The create_enum macro is used to have a sintactic sugar way to associate raw strings to the the typed enum variant.

* **Metadata**: `ColumnInfo` tracks the count of these types and stores unique variants to facilitate categorical Enum generation.

## Main structure of the generated code
This is the example for the iris dataset:
```
sepal length (cm),sepal width (cm),petal length (cm),petal width (cm),target
5.1,3.5,1.4,0.2,Iris-setosa
4.9,3.0,1.4,0.2,Iris-setosa
4.7,3.2,1.3,0.2,Iris-setosa
4.6,3.1,1.5,0.2,Iris-setosa
```
Rust generated code:
```rust
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
```
Each enum used to represent the csv value have a Null variant.
