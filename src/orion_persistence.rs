use serde_json::{self, Map, Value};
use std::fs::File;
use std::path::PathBuf;

use crate::constants::RoutingColumns;
use crate::orion_state::OrionState;

pub struct OrionPersistence {
    file: PathBuf,
}

impl OrionPersistence {
    const FILE_FORMAT_ERROR_MESSAGE: &str = "Orion 32 Routing JSON file is malformed.";
    const TEXT_COLOR: &str = "#ffffff";

    pub(crate) fn new(file: PathBuf) -> Self {
        Self { file }
    }

    pub(crate) fn encode(&self, orion_state: &OrionState) {
        let routing_in = Value::Array(
            (0..7)
                .map(|row| {
                    Value::Array(
                        (0..32)
                            .map(|column| {
                                let route = row + column * RoutingColumns::STEP;
                                let value = orion_state.label_for_route(route, false);
                                let enabled =
                                    (column as usize) < orion_state.target_counts[row as usize];
                                let bg_color = orion_state.hex_color_for_route(route, false);

                                serde_json::json!({
                                    "value": value,
                                    "src_col": column,
                                    "src_row": row,
                                    "enabled": enabled,
                                    "color": Self::TEXT_COLOR,
                                    "bg_color": bg_color,
                                })
                            })
                            .collect(),
                    )
                })
                .collect(),
        );

        let value = serde_json::json!({ "routing_in": routing_in });

        let writer = File::create(&self.file)
            .unwrap_or_else(|error| panic!("Failed to create {}: {}", self.file.display(), error));

        serde_json::to_writer_pretty(writer, &value)
            .unwrap_or_else(|error| panic!("Failed to encode {}: {}", self.file.display(), error));
    }

    pub(crate) fn decode(&self) -> ([[String; 32]; 7], [[u8; 32]; 7]) {
        let reader = File::open(&self.file)
            .unwrap_or_else(|error| panic!("Failed to open {}: {}", self.file.display(), error));

        let value: Value = serde_json::from_reader(reader)
            .unwrap_or_else(|error| panic!("Failed to decode {}: {}", self.file.display(), error));

        let root = Self::object(&value);
        let input = Self::array(Self::field(root, "routing_in"));

        let labels = std::array::from_fn(|row| {
            let group = Self::array(&input[row]);

            std::array::from_fn(|column| {
                let item = Self::object(&group[column]);
                Self::string(Self::field(item, "value")).to_string()
            })
        });

        let routes = std::array::from_fn(|row| {
            let group = Self::array(&input[row]);

            std::array::from_fn(|column| {
                let item = Self::object(&group[column]);
                let src_col = Self::i64_value(Self::field(item, "src_col"));
                let src_row = Self::i64_value(Self::field(item, "src_row"));

                (src_col * RoutingColumns::STEP as i64 + src_row) as u8
            })
        });

        (labels, routes)
    }

    fn field<'a>(object: &'a Map<String, Value>, key: &str) -> &'a Value {
        object
            .get(key)
            .unwrap_or_else(|| panic!("{}", Self::FILE_FORMAT_ERROR_MESSAGE))
    }

    fn array(value: &Value) -> &Vec<Value> {
        let Value::Array(value) = value else {
            panic!("{}", Self::FILE_FORMAT_ERROR_MESSAGE);
        };

        value
    }

    fn object(value: &Value) -> &Map<String, Value> {
        let Value::Object(value) = value else {
            panic!("{}", Self::FILE_FORMAT_ERROR_MESSAGE);
        };

        value
    }

    fn string(value: &Value) -> &str {
        let Value::String(value) = value else {
            panic!("{}", Self::FILE_FORMAT_ERROR_MESSAGE);
        };

        value
    }

    fn i64_value(value: &Value) -> i64 {
        value
            .as_i64()
            .unwrap_or_else(|| panic!("{}", Self::FILE_FORMAT_ERROR_MESSAGE))
    }
}
