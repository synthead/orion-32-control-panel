use serde_json;
use std::collections::HashMap;
use std::fs::File;
use std::path::PathBuf;

use crate::constants::RoutingColumns;
use crate::orion_state::OrionState;

pub struct OrionPersistence {
    file: PathBuf,
}

impl OrionPersistence {
    pub(crate) fn new(file: PathBuf) -> Self {
        Self { file }
    }

    pub(crate) fn encode(&self, orion_state: &OrionState) {
        let mut data: HashMap<String, [[String; 32]; 7]> = if self.file.exists() {
            let reader = File::open(&self.file).unwrap_or_else(|error| {
                panic!("Failed to open {}: {}", self.file.display(), error)
            });

            serde_json::from_reader(reader).unwrap_or_else(|error| {
                panic!("Failed to decode {}: {}", self.file.display(), error)
            })
        } else {
            HashMap::new()
        };

        let labels = std::array::from_fn(|row| {
            std::array::from_fn(|column| {
                let route = row as u8 + column as u8 * RoutingColumns::STEP;
                orion_state.label_for_route(route as u8, false).to_string()
            })
        });

        data.insert(Self::serial_key(&orion_state.serial), labels);

        let writer = File::create(&self.file)
            .unwrap_or_else(|error| panic!("Failed to create {}: {}", self.file.display(), error));

        serde_json::to_writer(writer, &data)
            .unwrap_or_else(|error| panic!("Failed to encode {}: {}", self.file.display(), error));
    }

    pub(crate) fn decode(&self, orion_state: &OrionState) -> Option<[[String; 32]; 7]> {
        let reader = File::open(&self.file)
            .unwrap_or_else(|error| panic!("Failed to open {}: {}", self.file.display(), error));

        let data: HashMap<String, [[String; 32]; 7]> = serde_json::from_reader(reader)
            .unwrap_or_else(|error| panic!("Failed to decode {}: {}", self.file.display(), error));

        data.get(&Self::serial_key(&orion_state.serial)).cloned()
    }

    fn serial_key(serial: &[u8; 13]) -> String {
        String::from_utf8_lossy(serial).into_owned()
    }
}
