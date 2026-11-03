use hidapi::HidApi;
use std::collections::HashMap;
use std::ffi::CString;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

use crate::orion_hid_device::OrionHidDevice;
use crate::orion_state::OrionStates;

pub(crate) struct OrionHidPoller {
    orion_states: Arc<Mutex<OrionStates>>,
    hid_write_rx: mpsc::Receiver<[u8; 64]>,

    hid_api: HidApi,
    orion_hid_devices: HashMap<CString, OrionHidDevice>,
}

impl OrionHidPoller {
    const USB_VENDOR: u16 = 0x23e5;
    const USB_PRODUCT: u16 = 0xa040;

    pub(crate) fn new(
        orion_states: Arc<Mutex<OrionStates>>,
        hid_write_rx: mpsc::Receiver<[u8; 64]>,
    ) -> Self {
        let hid_api = HidApi::new().unwrap_or_else(|error| {
            eprintln!("Failed to initialize HIDAPI: {error}");
            std::process::exit(1);
        });

        Self {
            orion_states,
            hid_write_rx,

            hid_api,
            orion_hid_devices: HashMap::new(),
        }
    }

    pub(crate) fn poll_and_update_state(&mut self) {
        if !self.try_connect() {
            self.discard_pending_writes();
            return;
        }

        let selected_hid_path = self.orion_states.lock().unwrap().selected_hid_path.clone();

        if let Some(selected_hid_path) = selected_hid_path {
            while let Ok(packet) = self.hid_write_rx.try_recv() {
                let write_success = self
                    .orion_hid_devices
                    .get(&selected_hid_path)
                    .is_some_and(|device| device.write_packet(&packet));

                if !write_success {
                    self.disconnect(&selected_hid_path);
                    self.discard_pending_writes();
                    break;
                }
            }
        } else {
            self.discard_pending_writes();
        }

        let device_paths: Vec<CString> = self.orion_hid_devices.keys().cloned().collect();

        for hid_path in device_paths {
            let poll_result = {
                let states = self.orion_states.lock().unwrap();

                match (
                    self.orion_hid_devices.get_mut(&hid_path),
                    states.orion_states.get(&hid_path),
                ) {
                    (Some(device), Some(orion_state)) => device.poll(orion_state),
                    _ => continue,
                }
            };

            match poll_result {
                Ok(Some(hid_data)) => {
                    let mut states = self.orion_states.lock().unwrap();

                    if let (Some(device), Some(orion_state)) = (
                        self.orion_hid_devices.get_mut(&hid_path),
                        states.orion_states.get_mut(&hid_path),
                    ) {
                        device.update_orion_state(&hid_data, orion_state);
                    }
                }
                Ok(None) => {}
                Err(()) => {
                    self.disconnect(&hid_path);
                }
            }
        }
    }

    fn try_connect(&mut self) -> bool {
        if let Err(error) = self.hid_api.refresh_devices() {
            eprintln!("Failed to refresh HID devices: {error}");
            return !self.orion_hid_devices.is_empty();
        }

        for device_info in self.hid_api.device_list() {
            if device_info.vendor_id() != Self::USB_VENDOR
                || device_info.product_id() != Self::USB_PRODUCT
            {
                continue;
            }

            let path = device_info.path().to_owned();

            if self.orion_hid_devices.contains_key(&path) {
                continue;
            }

            let hid_device = match device_info.open_device(&self.hid_api) {
                Ok(device) => device,
                Err(_) => continue,
            };

            self.orion_hid_devices
                .insert(path.clone(), OrionHidDevice::new(hid_device));

            let mut states = self.orion_states.lock().unwrap();

            states.orion_states.entry(path.clone()).or_default();

            if states.selected_hid_path.is_none() {
                states.selected_hid_path = Some(path);
            }
        }

        !self.orion_hid_devices.is_empty()
    }

    fn disconnect(&mut self, hid_path: &CString) {
        self.orion_hid_devices.remove(hid_path);

        let mut states = self.orion_states.lock().unwrap();

        states.orion_states.remove(hid_path);

        if states.selected_hid_path.as_ref() == Some(hid_path) {
            states.selected_hid_path = self.orion_hid_devices.keys().next().cloned();
        }
    }

    fn discard_pending_writes(&self) {
        while self.hid_write_rx.try_recv().is_ok() {}
    }
}
