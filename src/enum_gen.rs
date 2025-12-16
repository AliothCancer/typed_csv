use crate::{COLUMN_TYPE_ENUM_NAME, ColName, CsvAny, CsvDataset, SanitizedStr, dataset_info::{ColumnInfo, Variant}, sanitizer::sanitize_identifier};


#[macro_export]
macro_rules! create_enum {
    // --- 1. Caso Base: Input finito ---
    // Quando la lista di input `[]` è vuota, generiamo il codice finale.
    (@step $name:ident, ($($variants:ident),*), ($($arms:tt)*), []) => {
        
        
        #[derive(Debug, PartialEq, Clone, Copy)]
        pub enum $name {
            $($variants),*
        }

        impl std::str::FromStr for $name {
            type Err = String;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    // Inseriamo qui tutti i rami match accumulati
                    $($arms)*
                    _ => Err(format!("Valore non riconosciuto: '{}'", s)),
                }
            }
        }
        
    };

    // --- 2. Caso Complesso: "str1" | "str2" => Variante ---
    // Riconosce il pattern stringa/e => Identificatore
    (@step $name:ident, ($($vars:ident),*), ($($arms:tt)*), [ $($l:literal)|+ => $v:ident, $($rest:tt)* ]) => {
        create_enum!(
            @step 
            $name, 
            ($($vars,)* $v), // Aggiunge la variante alla lista
            ($($arms)* $($l)|+ => Ok($name::$v),), // Aggiunge il match arm personalizzato
            [ $($rest)* ] // Continua con il resto
        );
    };
    // Gestione dell'ultimo elemento (senza virgola finale) per il caso complesso
    (@step $name:ident, ($($vars:ident),*), ($($arms:tt)*), [ $($l:literal)|+ => $v:ident ]) => {
        create_enum!(@step $name, ($($vars,)* $v), ($($arms)* $($l)|+ => Ok($name::$v),), []);
    };

    // --- 3. Caso Semplice: Variante ---
    // Riconosce solo l'Identificatore (usa il nome stesso come stringa)
    (@step $name:ident, ($($vars:ident),*), ($($arms:tt)*), [ $v:ident, $($rest:tt)* ]) => {
        create_enum!(
            @step 
            $name, 
            ($($vars,)* $v), 
            ($($arms)* stringify!($v) => Ok($name::$v),), 
            [ $($rest)* ]
        );
    };
    // Gestione dell'ultimo elemento (senza virgola finale) per il caso semplice
    (@step $name:ident, ($($vars:ident),*), ($($arms:tt)*), [ $v:ident ]) => {
        create_enum!(@step $name, ($($vars,)* $v), ($($arms)* stringify!($v) => Ok($name::$v),), []);
    };

    // --- Entry Point ---
    // Inizializza gli accumulatori vuoti
    ($name:ident; $($input:tt)*) => {
        create_enum!(@step $name, (), (), [ $($input)* ]);
    };
}


pub fn generate_enums_from(dataset: &mut CsvDataset) -> String{
    
    let mut full_string = String::new();

    // I cloned because I need CsvDataset as mutable
    // to populate the field `info:Vec<ColumnInfo>`
    // but I also need a reference to `names` (another field of itself)
    let (value_names_view, info) = dataset.split_view_and_info();
    let col_name = value_names_view.names;
    let enums = col_name.iter().map(|col_name| {

        let mut col_info = ColumnInfo::new(value_names_view, &col_name.raw);


        if !col_info.unique_values.iter().any(|x| x.csvany == CsvAny::Null){
            let str = String::from("Null");
            col_info.unique_values.push(Variant{ raw: str.clone(), sanitized: str, csvany: CsvAny::Null});
        }

        info.push(col_info.clone());
        
        let unique_val_iter = col_info.unique_values.iter();

        
        let is_str = unique_val_iter.clone().all(|x| {
            matches!(x.csvany, CsvAny::Str(_) | CsvAny::Empty | CsvAny::Null)
        });
        let is_int = unique_val_iter.clone().all(|x| {
            matches!(x.csvany, CsvAny::Int(_) | CsvAny::Empty | CsvAny::Null)
        });
        let is_float = unique_val_iter.clone().all(|x| {
            matches!(x.csvany, CsvAny::Float(_) | CsvAny::Empty | CsvAny::Null)
        });
        
        if is_int{
            gen_int_enum(col_name)
        }else if is_float{
            gen_float_enum(col_name)
        }
        else if is_str{
            gen_str_enum(col_name, unique_val_iter)
        }else {
            println!("enum generation log: column `{}` contains numbers and strings", col_name.raw);
            gen_str_enum(col_name, unique_val_iter)
        }
        
    } + "\n\n").collect::<String>();

    
    let mut columns_enum = format!("enum {COLUMN_TYPE_ENUM_NAME}{{\n");

    col_name.iter().for_each(|col_name|{
        columns_enum.push_str(&format!("{},\n", col_name.sanitized.0));
    });
    columns_enum.push_str("}\n\n");

    let mut columns_enum_from_str = format!("\
impl std::str::FromStr for {COLUMN_TYPE_ENUM_NAME}{{
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {{ 
        match s{{
");
    col_name.iter().for_each(|col_name|{
        let ColName{raw, sanitized} = col_name;
        let SanitizedStr(sanitized) = sanitized; 
        columns_enum_from_str.push_str(&format!("\t\t\t\"{raw}\" => Ok({COLUMN_TYPE_ENUM_NAME}::{sanitized}),\n"));
    });
    // last case
    columns_enum_from_str.push_str("_ => Err(format!(\"Unknown string: '{}'\", s)),\n");
    columns_enum_from_str.push_str("}\n}\n}");

    full_string.push_str(&enums);
    full_string.push_str(&columns_enum);
    full_string.push_str(&columns_enum_from_str);
    full_string
}

fn gen_str_enum<'a>(col_name: &ColName, unique_values: impl Iterator<Item = &'a Variant>) -> String{
    let variants = unique_values
            .map(|var| match &var.csvany {
                CsvAny::Int(int) => sanitize_identifier(&int.to_string()),
                CsvAny::Str(str) => format!("\"{}\" => {}", str, sanitize_identifier(str)),
                CsvAny::Empty => "Empty".to_string(),
                CsvAny::Null => "Null".to_string(),
                CsvAny::Float(_) => panic!("Should not be used on float since they cannot represent categories"),
            } + ",\n")
            .collect::<String>();
        format!("create_enum!({};\n{variants});", col_name.sanitized.0)
}
fn gen_float_enum(col_name: &ColName) -> String {
    let name = &col_name.sanitized.0;
    format!("
    #[derive(Debug, Clone, Copy, PartialEq)]
    enum {name} {{ Float(f64), Null }}

    impl std::str::FromStr for {name}{{
        type Err = String;

        fn from_str(s: &str) -> Result<Self, Self::Err> {{ 
            let f = s.parse::<f64>().unwrap();
            Ok({name}::Float(f))
        }}
    }}")
}

fn gen_int_enum(col_name: &ColName) -> String {
    let name = &col_name.sanitized.0;
    format!("
    #[derive(Debug, Clone, Copy, PartialEq)]
    enum {name} {{ Int(i64), Null }}

    impl std::str::FromStr for {name}{{
        type Err = String;

        fn from_str(s: &str) -> Result<Self, Self::Err> {{ 
            let i = s.parse::<i64>().unwrap();
            Ok({name}::Int(i))
        }}
    }}
    ")
}