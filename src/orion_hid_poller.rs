use hidapi::HidApi;
use std::collections::HashMap;
use std::ffi::CString;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::orion_hid_device::OrionHidDevice;
use crate::orion_state::{OrionStateInitSteps, OrionStates};

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
            thread::sleep(Duration::from_millis(20));

            return;
        }

        let selected_hid_path = self.orion_states.lock().unwrap().selected_hid_path.clone();

        let Some(selected_hid_path) = selected_hid_path.as_ref() else {
            self.discard_pending_writes();
            return;
        };

        while let Ok(packet) = self.hid_write_rx.try_recv() {
            let write_success = self
                .orion_hid_devices
                .get(selected_hid_path)
                .is_some_and(|device| device.write_packet(&packet));

            if !write_success {
                self.disconnect(selected_hid_path);
                self.discard_pending_writes();
                return;
            }
        }

        let poll_result = {
            let states = self.orion_states.lock().unwrap();

            match (
                self.orion_hid_devices.get_mut(selected_hid_path),
                states.orion_states.get(selected_hid_path),
            ) {
                (Some(device), Some(_)) => {
                    if device.ensure_init() {
                        device.read_packet()
                    } else {
                        Err(())
                    }
                }
                _ => return,
            }
        };

        match poll_result {
            Ok(Some(hid_data)) => {
                let mut states = self.orion_states.lock().unwrap();

                if let (Some(device), Some(orion_state)) = (
                    self.orion_hid_devices.get_mut(selected_hid_path),
                    states.orion_states.get_mut(selected_hid_path),
                ) {
                    device.update_orion_state(&hid_data, orion_state);
                }
            }
            Ok(None) => {}
            Err(()) => {
                self.disconnect(selected_hid_path);
                self.discard_pending_writes();
            }
        }
    }

    fn try_connect(&mut self) -> bool {
        if let Err(error) = self.hid_api.refresh_devices() {
            eprintln!("Failed to refresh HID devices: {error}");
            return !self.orion_hid_devices.is_empty();
        }

        let available_paths: Vec<CString> = self
            .hid_api
            .device_list()
            .filter(|device_info| {
                device_info.vendor_id() == Self::USB_VENDOR
                    && device_info.product_id() == Self::USB_PRODUCT
            })
            .map(|device_info| device_info.path().to_owned())
            .collect();

        let selected_hid_path = {
            let mut states = self.orion_states.lock().unwrap();

            states
                .orion_states
                .retain(|path, _| available_paths.contains(path));

            for path in &available_paths {
                states.orion_states.entry(path.clone()).or_default();
            }

            if states
                .selected_hid_path
                .as_ref()
                .is_none_or(|path| !available_paths.contains(path))
            {
                states.selected_hid_path = available_paths.first().cloned();
            }

            states.selected_hid_path.clone()
        };

        self.orion_hid_devices
            .retain(|path, _| available_paths.contains(path));

        let paths_to_open: Vec<CString> = {
            let states = self.orion_states.lock().unwrap();

            available_paths
                .iter()
                .filter(|path| {
                    if self.orion_hid_devices.contains_key(*path) {
                        return false;
                    }

                    if Some(*path) == selected_hid_path.as_ref() {
                        return true;
                    }

                    states.orion_states.get(*path).is_some_and(|orion_state| {
                        orion_state.inits_completed != OrionStateInitSteps::ALL
                    })
                })
                .cloned()
                .collect()
        };

        for path in paths_to_open {
            let hid_device = self
                .hid_api
                .device_list()
                .find(|device_info| path == device_info.path())
                .and_then(|device_info| device_info.open_device(&self.hid_api).ok());

            let Some(hid_device) = hid_device else {
                continue;
            };

            let mut device = OrionHidDevice::new(hid_device);

            if Some(&path) == selected_hid_path.as_ref() {
                self.orion_hid_devices.insert(path, device);
                continue;
            }

            loop {
                if !device.ensure_init() {
                    break;
                }

                match device.read_packet() {
                    Ok(Some(hid_data)) => {
                        let initialized = {
                            let mut states = self.orion_states.lock().unwrap();

                            if let Some(orion_state) = states.orion_states.get_mut(&path) {
                                device.update_orion_state(&hid_data, orion_state);

                                orion_state.inits_completed == OrionStateInitSteps::ALL
                            } else {
                                false
                            }
                        };

                        if initialized {
                            break;
                        }
                    }
                    Ok(None) => {}
                    Err(()) => break,
                }
            }
        }

        selected_hid_path
            .as_ref()
            .is_some_and(|path| self.orion_hid_devices.contains_key(path))
    }

    fn disconnect(&mut self, hid_path: &CString) {
        self.orion_hid_devices.remove(hid_path);

        let mut states = self.orion_states.lock().unwrap();

        states.orion_states.remove(hid_path);

        if states.selected_hid_path.as_ref() == Some(hid_path) {
            states.selected_hid_path = None;
        }
    }

    fn discard_pending_writes(&self) {
        while self.hid_write_rx.try_recv().is_ok() {}
    }
}
