use serde_pickle::{self, value::HashableValue, Value};
use std::collections::BTreeMap;
use std::fs::File;
use std::path::PathBuf;

use crate::orion_state::OrionState;
use crate::routing_constants::RoutingColumns;

pub struct OrionFile {
    file: PathBuf,
}

impl OrionFile {
    const FILE_FORMAT_ERROR_MESSAGE: &str = "Orion 32 Routing File is malformed.";
    const TEXT_COLOR: &str = "#ffffff";

    pub(crate) fn new(file: PathBuf) -> Self {
        Self { file }
    }

    pub(crate) fn encode(&self, orion_state: &OrionState) {
        let input = Value::List(
            orion_state
                .labels
                .iter()
                .enumerate()
                .map(|(row, labels)| {
                    Value::List(
                        labels
                            .iter()
                            .enumerate()
                            .map(|(column, label)| {
                                let bg_color_string =
                                    orion_state.hex_color_for_route(row as u8, false);

                                let value = Value::String(label.clone());
                                let enabled = Value::Bool(column < orion_state.source_counts[row]);
                                let color = Value::String(Self::TEXT_COLOR.to_string());
                                let bg_color = Value::String(bg_color_string);

                                Self::make_dict([
                                    ("value", value),
                                    ("src_row", Value::None),
                                    ("enabled", enabled),
                                    ("src_col", Value::None),
                                    ("color", color),
                                    ("bg_color", bg_color),
                                ])
                            })
                            .collect(),
                    )
                })
                .collect(),
        );

        let output = Value::List(
            orion_state
                .routes
                .iter()
                .enumerate()
                .map(|(target_row, routes)| {
                    Value::List(
                        routes
                            .iter()
                            .enumerate()
                            .map(|(target_column, route)| {
                                let source_column = *route / RoutingColumns::STEP;
                                let source_row = *route % RoutingColumns::STEP;
                                println!("{:02X} {:02X}", source_row, source_column);
                                let bg_color_string =
                                    orion_state.hex_color_for_route(*route, false);

                                let value =
                                    Value::String(orion_state.label_for_route(*route, false));
                                let source_row = Value::I64(source_row as i64);
                                let enabled = Value::Bool(
                                    target_column < orion_state.target_counts[target_row],
                                );
                                let source_column = Value::I64(source_column as i64);
                                let color = Value::String(Self::TEXT_COLOR.to_string());
                                let bg_color = Value::String(bg_color_string);

                                Self::make_dict([
                                    ("value", value),
                                    ("src_row", source_row),
                                    ("enabled", enabled),
                                    ("src_col", source_column),
                                    ("color", color),
                                    ("bg_color", bg_color),
                                ])
                            })
                            .collect(),
                    )
                })
                .collect(),
        );

        let grids = Self::make_dict([("input", input), ("output", output)]);

        let value = Self::make_dict([("grids", grids), ("version", Value::I64(2))]);

        let mut writer = File::create(&self.file)
            .unwrap_or_else(|error| panic!("Failed to create {}: {}", self.file.display(), error));

        serde_pickle::value_to_writer(&mut writer, &value, serde_pickle::SerOptions::new())
            .unwrap_or_else(|error| panic!("Failed to encode {}: {}", self.file.display(), error));
    }

    pub(crate) fn decode(&self) -> ([[String; 32]; 7], [[u8; 32]; 7]) {
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

                (src_col * RoutingColumns::STEP as i64 + src_row) as u8
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

    fn make_dict<const N: usize>(fields: [(&str, Value); N]) -> Value {
        Value::Dict(
            fields
                .into_iter()
                .map(|(key, value)| (HashableValue::String(key.to_string()), value))
                .collect(),
        )
    }
}
