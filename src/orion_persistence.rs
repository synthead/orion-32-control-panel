use serde_json::{Value, json};
use std::collections::HashMap;
use std::fs::File;
use std::sync::{Arc, Mutex};

use crate::constants::RoutingColumns;
use crate::orion_paths::OrionPaths;
use crate::orion_state::OrionState;

pub struct OrionPersistence {
    orion_paths: Arc<Mutex<OrionPaths>>,
}

impl OrionPersistence {
    pub(crate) fn new(orion_paths: Arc<Mutex<OrionPaths>>) -> Self {
        Self { orion_paths }
    }

    pub(crate) fn read(&self, serial: &str) -> Option<[[String; 32]; 7]> {
        self.read_route_labels().remove(serial)
    }

    pub(crate) fn write(&self, orion_state: &OrionState) {
        let file = self.orion_paths.lock().unwrap().persistence_file_path();
        let mut route_labels = self.read_route_labels();

        let labels = std::array::from_fn(|row| {
            std::array::from_fn(|column| {
                let route = row as u8 + column as u8 * RoutingColumns::STEP;
                orion_state.label_for_route(route, false).to_string()
            })
        });

        let serial = Self::serial_key(&orion_state.serial);

        route_labels.insert(serial.clone(), labels);

        let value = json!({
            "selected_serial": serial,
            "route_labels": route_labels,
        });

        let writer = File::create(&file)
            .unwrap_or_else(|error| panic!("Failed to create {}: {}", file.display(), error));

        serde_json::to_writer(writer, &value)
            .unwrap_or_else(|error| panic!("Failed to encode {}: {}", file.display(), error));
    }

    pub(crate) fn read_selected_serial(&self) -> Option<String> {
        self.read_json_data()
            .get("selected_serial")
            .and_then(Value::as_str)
            .map(str::to_owned)
    }

    fn read_json_data(&self) -> Value {
        let file = self.orion_paths.lock().unwrap().persistence_file_path();

        if !file.exists() {
            return Value::Null;
        }

        let reader = File::open(&file)
            .unwrap_or_else(|error| panic!("Failed to open {}: {}", file.display(), error));

        serde_json::from_reader(reader)
            .unwrap_or_else(|error| panic!("Failed to decode {}: {}", file.display(), error))
    }

    fn read_route_labels(&self) -> HashMap<String, [[String; 32]; 7]> {
        match self.read_json_data().get("route_labels") {
            Some(labels) => serde_json::from_value(labels.clone()).unwrap_or_else(|error| {
                panic!("Failed to decode route labels in the persistence file: {error}")
            }),
            None => HashMap::new(),
        }
    }

    fn serial_key(serial: &[u8; 13]) -> String {
        String::from_utf8_lossy(serial).into_owned()
    }
}
