use serde_json::{Value, json};
use std::collections::HashMap;
use std::fs::File;
use std::sync::{Arc, Mutex};

use crate::constants::{RoutingColumns, RoutingLabels};
use crate::orion_paths::OrionPaths;
use crate::orion_state::OrionState;

type Labels = [[String; 32]; 7];
type PresetLabels = HashMap<String, Labels>;

pub struct OrionPersistence {
    orion_paths: Arc<Mutex<OrionPaths>>,
}

impl OrionPersistence {
    const PRESET_KEYS: [&str; 6] = [
        "no_preset",
        "preset_1",
        "preset_2",
        "preset_3",
        "preset_4",
        "preset_5",
    ];

    pub(crate) fn new(orion_paths: Arc<Mutex<OrionPaths>>) -> Self {
        Self { orion_paths }
    }

    pub(crate) fn read(&self, orion_state: &mut OrionState) {
        let key = Self::PRESET_KEYS[(orion_state.preset_standby & 0x07) as usize];
        let serial = Self::serial_key(&orion_state.serial);

        let labels = self
            .read_route_labels()
            .remove(&serial)
            .and_then(|mut presets| presets.remove(key));

        if let Some(labels) = labels {
            orion_state.labels = Arc::new(labels);
        }
    }

    pub(crate) fn write(&self, orion_state: &OrionState, preset: u8) {
        let key = Self::PRESET_KEYS[preset as usize];

        let mut route_labels = self.read_route_labels();

        let labels = std::array::from_fn(|row| {
            std::array::from_fn(|column| {
                let route = row as u8 + column as u8 * RoutingColumns::STEP;
                orion_state.label_for_route(route, false).to_string()
            })
        });

        let serial = Self::serial_key(&orion_state.serial);

        Self::populate_defaults(&mut route_labels, &serial);

        route_labels
            .get_mut(&serial)
            .unwrap()
            .insert(key.to_owned(), labels);

        self.write_json_data(&serial, &route_labels);
    }

    pub(crate) fn read_selected_serial(&self) -> Option<String> {
        self.read_json_data()
            .get("selected_serial")
            .and_then(Value::as_str)
            .map(str::to_owned)
    }

    pub(crate) fn write_selected_serial(&self, serial: &str) {
        let mut route_labels = self.read_route_labels();

        Self::populate_defaults(&mut route_labels, serial);

        self.write_json_data(serial, &route_labels);
    }

    fn populate_defaults(route_labels: &mut HashMap<String, PresetLabels>, serial: &str) {
        if route_labels.contains_key(serial) {
            return;
        }

        let default_labels: Labels =
            RoutingLabels::DEFAULT_LABEL_NUMBERS.map(|row| row.map(|number| number.to_string()));

        let presets = Self::PRESET_KEYS
            .iter()
            .map(|key| (key.to_string(), default_labels.clone()))
            .collect();

        route_labels.insert(serial.to_owned(), presets);
    }

    fn read_json_data(&self) -> Value {
        let file = self.orion_paths.lock().unwrap().persistence_file_path();

        File::open(&file)
            .ok()
            .and_then(|reader| serde_json::from_reader(reader).ok())
            .unwrap_or(Value::Null)
    }

    fn write_json_data(&self, selected_serial: &str, route_labels: &HashMap<String, PresetLabels>) {
        let file = self.orion_paths.lock().unwrap().persistence_file_path();

        let value = json!({
            "selected_serial": selected_serial,
            "route_labels": route_labels,
        });

        let writer = File::create(&file)
            .unwrap_or_else(|error| panic!("Failed to create {}: {}", file.display(), error));

        serde_json::to_writer(writer, &value)
            .unwrap_or_else(|error| panic!("Failed to encode {}: {}", file.display(), error));
    }

    fn read_route_labels(&self) -> HashMap<String, PresetLabels> {
        self.read_json_data()
            .get("route_labels")
            .and_then(|labels| serde_json::from_value(labels.clone()).ok())
            .unwrap_or_default()
    }

    fn serial_key(serial: &[u8; 13]) -> String {
        String::from_utf8_lossy(serial).into_owned()
    }
}
