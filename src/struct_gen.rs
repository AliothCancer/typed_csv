#![allow(clippy::uninlined_format_args)]

use crate::{
    COLUMN_TYPE_ENUM_NAME, ColName, CsvDataset, MAIN_STRUCT_NAME, SanitizedStr, csv_type::CsvAny, dataset_info::{ColumnInfo, Variant}
};

/// It generates a struct named `CsvDataFrame` which
/// contains all `Vec<T>` where T is the generated enums
/// for each columns
pub fn gen_struct(dataset: &CsvDataset) -> String {
    let mut final_str = format!("pub struct {MAIN_STRUCT_NAME}{{\n");

    // final_str.push_str(&format!("\tpub columns: Vec<{COLUMN_TYPE_ENUM_NAME}>,\n"));
    dataset.names.iter().for_each(|name| {
        final_str.push_str(&format!(
            "\tpub {}: {},\n",
            name.sanitized.0.to_lowercase(),
            COLUMN_TYPE_ENUM_NAME
        ));
    });
    final_str.push('}');

    let impl_str_open = format!(
        "impl {MAIN_STRUCT_NAME}{{
"
    );
    let new_method = gen_new_method(&dataset.names, &dataset.info);
    let column_list_method = gen_column_list_method(&dataset.names);
    let impl_str_close = '}';

    final_str.push_str(&impl_str_open);
    final_str.push_str(&new_method);
    final_str.push_str(&column_list_method);
    final_str.push(impl_str_close);

    final_str
}

fn gen_column_list_method(col_names: &[ColName]) -> String {
    let mut number_of_cols = 0;
    let columns = col_names
        .iter()
        .map(|x| {
            number_of_cols += 1;
            format!("&self.{},", x.sanitized.0)
        })
        .collect::<String>();
    format!(
        "\
    pub fn get_columns(&self)-> [&{COLUMN_TYPE_ENUM_NAME};{number_of_cols}] {{
        [{columns}]
    }}"
    )
}

fn gen_new_method(col_names: &[ColName], cols_info: &[ColumnInfo]) -> String {
    let vecs_of_enums = cols_info
        .iter()
        .map(|col_info| gen_vec_of_enums(&col_info.column_name, &col_info.unique_values) + "\n\n")
        .collect::<String>();

    let fields_list = col_names
        .iter()
        .map(|colname| format!("{},\n\t\t\t", colname.sanitized.0.to_lowercase()))
        .collect::<String>();
    format!(
        "\
    pub fn new(dataset: &csv_deserializer::CsvDataset) -> Self{{
        {vecs_of_enums}

        {MAIN_STRUCT_NAME}{{
            {fields_list}
        }}
    }}
"
    )
}

fn gen_vec_of_enums(col_name: &ColName, unique_values: &[Variant]) -> String {
    let ColName {
        raw: _raw,
        sanitized,
    } = col_name;
    let SanitizedStr(sanitized) = sanitized;
    let sanitized_lower = sanitized.to_lowercase();
    let mut float_case_already_written = false;
    let mut int_case_already_written = false;
    let mut str_case_already_written = false;
    let match_arms = unique_values
        .iter()
        .filter_map(|v| match &v.csvany {
            CsvAny::Str(_) if !str_case_already_written => {
                str_case_already_written = true;
                Some(format!(
                    "csv_deserializer::csv_type::CsvAny::Str(s) => {sanitized}::from_str(s).unwrap(),\n"
                ))
            }
            CsvAny::Int(_) if !int_case_already_written => {
                int_case_already_written = true;
                Some(format!(
                    "csv_deserializer::csv_type::CsvAny::Int(i) => {sanitized}::Int(*i),\n"
                ))
            }
            CsvAny::Float(_) if !float_case_already_written => {
                float_case_already_written = true;
                Some(format!(
                    "csv_deserializer::csv_type::CsvAny::Float(f) => {sanitized}::Float(*f),\n"
                ))
            }
            CsvAny::Null => Some(format!(
                "csv_deserializer::csv_type::CsvAny::Null => {sanitized}::Null,\n"
            )),
            CsvAny::Empty => Some(format!(
                "csv_deserializer::csv_type::CsvAny::Empty => {sanitized}::Null,\n"
            )),
            _ => None,
        })
        .collect::<String>();
    format!(
        "\
let (index, _) = dataset
            .names
            .iter()
            .enumerate()
            .find(|(index, cl)| &cl.sanitized.0 == \"{sanitized}\")
            .unwrap();
let {sanitized_lower} = {COLUMN_TYPE_ENUM_NAME}::{sanitized}(dataset.values[index].iter().map(|val| match val{{
    {match_arms}
    _ => panic!(),
}}).collect::<Vec<{sanitized}>>());
    "
    )
}
