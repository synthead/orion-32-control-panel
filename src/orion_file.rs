use serde_pickle::{self, value::HashableValue, Value};
use std::collections::BTreeMap;
use std::fs::File;
use std::path::PathBuf;

struct OrionFileRouteCodes;

impl OrionFileRouteCodes {
    const ADC_DAC: u8 = 0x00;
    const USB: u8 = 0x01;
    const MADI_1: u8 = 0x02;
    const MADI_2: u8 = 0x03;
    const ADAT: u8 = 0x04;
    const SPDIF: u8 = 0x05;
    const MIX_LR: u8 = 0x06;
    const OSC_1: u8 = 0x07;
    const OSC_2: u8 = 0x0f;
    const MUTE: u8 = 0x17;
    const COLUMN: u8 = 0x08;
}

struct OrionFileRouteColors;

impl OrionFileRouteColors {
    const ADC_DAC: &str = "#20a4f7";
    const USB: &str = "#0073ff";
    const MADI: &str = "#ffb800";
    const ADAT: &str = "#ff0037";
    const SPDIF: &str = "#d92494";
    const MIX_LR: &str = "#72ab20";
    const OSC_1: &str = "#3890a7";
    const OSC_2: &str = "#9038a7";
    const MUTE: &str = "#7b7b7b";
    const TEXT: &str = "#ffffff";
}

pub struct OrionFile {
    file: PathBuf,
}

impl OrionFile {
    const FILE_FORMAT_ERROR_MESSAGE: &str = "Orion 32 Routing File is malformed.";

    pub(crate) fn new(file: PathBuf) -> Self {
        Self { file }
    }

    pub(crate) fn encode(
        &self,
        all_labels: &[[String; 32]; 7],
        all_routes: &[[u8; 32]; 7],
        source_counts: &[usize; 7],
        target_counts: &[usize; 7],
    ) {
        let input = Value::List(
            all_labels
                .iter()
                .enumerate()
                .map(|(row, labels)| {
                    Value::List(
                        labels
                            .iter()
                            .enumerate()
                            .map(|(column, label)| {
                                let bg_color_str = Self::color_for_route(row as u8, source_counts);

                                let value = Value::String(label.clone());
                                let enabled = Value::Bool(column < source_counts[row]);
                                let color = Value::String(OrionFileRouteColors::TEXT.to_string());
                                let bg_color = Value::String(bg_color_str.to_string());

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
            all_routes
                .iter()
                .enumerate()
                .map(|(target_row, routes)| {
                    Value::List(
                        routes
                            .iter()
                            .enumerate()
                            .map(|(target_column, route)| {
                                let source_column = *route / OrionFileRouteCodes::COLUMN;
                                let source_row = *route % OrionFileRouteCodes::COLUMN;
                                let bg_color_str = Self::color_for_route(*route, source_counts);

                                let value =
                                    Value::String(all_labels[target_row][target_column].clone());
                                let source_row = Value::I64(source_row as i64);
                                let enabled =
                                    Value::Bool(target_column < target_counts[target_row]);
                                let source_column = Value::I64(source_column as i64);
                                let color = Value::String(OrionFileRouteColors::TEXT.to_string());
                                let bg_color = Value::String(bg_color_str.to_string());

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

                (src_col * OrionFileRouteCodes::COLUMN as i64 + src_row) as u8
            })
        });

        (labels, routes)
    }

    fn color_for_route(route: u8, source_counts: &[usize; 7]) -> &'static str {
        match route {
            OrionFileRouteCodes::OSC_1 => OrionFileRouteColors::OSC_1,
            OrionFileRouteCodes::OSC_2 => OrionFileRouteColors::OSC_2,
            OrionFileRouteCodes::MUTE => OrionFileRouteColors::MUTE,

            _ => {
                let source_row = route % OrionFileRouteCodes::COLUMN;
                let source_column = route / OrionFileRouteCodes::COLUMN;

                if source_row == OrionFileRouteCodes::USB
                    && source_column as usize >= source_counts[1]
                {
                    return OrionFileRouteColors::MUTE;
                }

                match source_row {
                    OrionFileRouteCodes::ADC_DAC => OrionFileRouteColors::ADC_DAC,
                    OrionFileRouteCodes::USB => OrionFileRouteColors::USB,
                    OrionFileRouteCodes::MADI_1 => OrionFileRouteColors::MADI,
                    OrionFileRouteCodes::MADI_2 => OrionFileRouteColors::MADI,
                    OrionFileRouteCodes::ADAT => OrionFileRouteColors::ADAT,
                    OrionFileRouteCodes::SPDIF => OrionFileRouteColors::SPDIF,
                    OrionFileRouteCodes::MIX_LR => OrionFileRouteColors::MIX_LR,

                    _ => OrionFileRouteColors::MUTE,
                }
            }
        }
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
