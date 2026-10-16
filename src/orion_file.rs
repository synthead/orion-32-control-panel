use serde_pickle::{self, value::HashableValue, Value};
use std::collections::BTreeMap;
use std::fs::File;
use std::path::PathBuf;

pub struct OrionFile {
    file: PathBuf,
}

impl OrionFile {
    const FILE_FORMAT_ERROR_MESSAGE: &str = "Orion 32 Routing File is malformed.";

    pub fn new(file: PathBuf) -> Self {
        Self { file }
    }

    pub fn decode(&self) -> ([[String; 32]; 7], [[u8; 32]; 7]) {
        let reader = File::open(&self.file)
            .unwrap_or_else(|error| panic!("Failed to open {}: {}", self.file.display(), error));
        let value = serde_pickle::value_from_reader(reader, Default::default())
            .unwrap_or_else(|error| panic!("Failed to decode {}: {}", self.file.display(), error));
        let grids = Self::dict(Self::field(Self::dict(&value), "grids"));

        let input = Self::list(Self::field(grids, "input"));
        let labels = std::array::from_fn(|row| {
            let group = Self::list(&input[row]);

            std::array::from_fn(|col| {
                let item = Self::dict(&group[col]);
                let label = Self::string(Self::field(item, "value"));

                label.clone()
            })
        });

        let output = Self::list(Self::field(grids, "output"));
        let routes = std::array::from_fn(|row| {
            let group = Self::list(&output[row]);

            std::array::from_fn(|col| {
                let item = Self::dict(&group[col]);
                let src_col = Self::i64_value(Self::field(item, "src_col"));
                let src_row = Self::i64_value(Self::field(item, "src_row"));

                (src_col * 0x08 + src_row) as u8
            })
        });

        (labels, routes)
    }

    fn field<'a>(dict: &'a BTreeMap<HashableValue, Value>, key: &str) -> &'a Value {
        &dict[&HashableValue::String(key.to_string())]
    }

    fn list(value: &Value) -> &Vec<Value> {
        let Value::List(value) = value else {
            panic!("{}", Self::FILE_FORMAT_ERROR_MESSAGE);
        };

        value
    }

    fn dict(value: &Value) -> &BTreeMap<HashableValue, Value> {
        let Value::Dict(value) = value else {
            panic!("{}", Self::FILE_FORMAT_ERROR_MESSAGE);
        };

        value
    }

    fn string(value: &Value) -> &String {
        let Value::String(value) = value else {
            panic!("{}", Self::FILE_FORMAT_ERROR_MESSAGE);
        };

        value
    }

    fn i64_value(value: &Value) -> i64 {
        let Value::I64(value) = value else {
            panic!("{}", Self::FILE_FORMAT_ERROR_MESSAGE);
        };

        *value
    }
}
