// mod csv_types;
// use csv_types::*;


use clap::Parser;

use std::{error::Error, fmt, fs::File, path::PathBuf};

use csv_deserializer::{
    csv_dataset::CsvDataset, NullValues, enum_gen::generate_enums_from, struct_gen::gen_struct,
};

/// Print to stdout the code generation for the provided `CsvDataset`
fn print_csv_rust_code(dataset: &mut CsvDataset) {
    let enums = generate_enums_from(dataset);
    let struc = gen_struct(dataset);
    let import = gen_imports();
    println!("#![allow(unused,non_snake_case,non_camel_case_types)]{import}\n{enums}\n{struc}");
}
fn gen_imports()-> String{
    String::from("\
use csv_deserializer::{create_enum, csv_dataset::CsvDataset, csv_types::CsvAny};
use std::str::FromStr;
\n")
}

#[derive(Parser)]
#[command(author = "AliothCancer", version)]
#[derive(Debug)]
struct Cli {
    #[arg(short = 'i', long = "input-file", value_name = "input_file", value_parser=custom_csv_path_validator)]
    input_file: PathBuf,
    #[arg(short = 'n', long = "null-values", value_name = "a,b,..")]
    null_values: Option<String>
}

fn main() -> Result<(), Box<dyn Error>> {
    let Cli { input_file, null_values } = Cli::parse();
    let file = File::open(input_file)?;
    let rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(file);
    let possible_nulls = match &null_values{
        Some(s) => s.split(',').map(str::trim).collect::<Vec<&str>>(),
        None => Vec::new(),
    };

    let mut dataset = CsvDataset::new(rdr, NullValues(possible_nulls));
    print_csv_rust_code(&mut dataset);
    Ok(())
}


#[derive(Debug)]
pub enum LocalCsvError {
    PathUnreachable(String), // Permission denied, etc.
    PathNotExists,
    NotAFile, // Flattens Exist::NotAFile
    NotACsv,  // Flattens IsFile::NotACsv
    MissingExtension,
}
impl Error for LocalCsvError {}

impl fmt::Display for LocalCsvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PathNotExists => write!(f, "The path provided does not exist."),
            Self::PathUnreachable(e) => write!(f, "System access error: {e}"),
            Self::NotAFile => write!(
                f,
                "The path exists but it is not a file (is it a directory?)."
            ),
            Self::NotACsv => write!(f, "The file extension suggests this is not a CSV."),
            Self::MissingExtension => write!(f, "Extension file is missing, should be .csv"),
        }
    }
}

fn custom_csv_path_validator(path_str: &str) -> Result<PathBuf, LocalCsvError> {
    let path = PathBuf::from(path_str);

    // 1. Check Existence / Reachability
    match path.try_exists() {
        Ok(true) => {} // pass to next checks
        Ok(false) => return Err(LocalCsvError::PathNotExists),
        Err(e) => return Err(LocalCsvError::PathUnreachable(e.to_string())),
    }

    // 2. Check if File
    if !path.is_file() {
        return Err(LocalCsvError::NotAFile);
    }

    // 3. Check Extension
    if let Some(ext) = path.extension() {
        if ext == "csv" {
            Ok(path)
        } else {
            Err(LocalCsvError::NotACsv)
        }
    } else {
        Err(LocalCsvError::MissingExtension)
    }
}
