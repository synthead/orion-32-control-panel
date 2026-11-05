use hidapi::HidApi;
use std::collections::HashMap;
use std::ffi::CString;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use crate::orion_hid_device::OrionHidDevice;
use crate::orion_persistence::OrionPersistence;
use crate::orion_state::{OrionStateInitSteps, OrionStates};

pub(crate) struct OrionHidPoller {
    orion_states: Arc<Mutex<OrionStates>>,
    persistence: Arc<Mutex<OrionPersistence>>,
    hid_write_rx: mpsc::Receiver<[u8; 64]>,

    hid_api: HidApi,
    orion_hid_devices: HashMap<CString, OrionHidDevice>,

    last_device_refresh: Option<Instant>,
    last_selected_hid_path: Option<CString>,
}

impl OrionHidPoller {
    const USB_VENDOR: u16 = 0x23e5;
    const USB_PRODUCT: u16 = 0xa040;

    const DEVICE_REFRESH_INTERVAL: Duration = Duration::from_secs(1);

    pub(crate) fn new(
        orion_states: Arc<Mutex<OrionStates>>,
        persistence: Arc<Mutex<OrionPersistence>>,
        hid_write_rx: mpsc::Receiver<[u8; 64]>,
    ) -> Self {
        let hid_api = HidApi::new().unwrap_or_else(|error| {
            eprintln!("Failed to initialize HIDAPI: {error}");
            std::process::exit(1);
        });

        Self {
            orion_states,
            persistence,
            hid_write_rx,

            hid_api,
            orion_hid_devices: HashMap::new(),

            last_device_refresh: None,
            last_selected_hid_path: None,
        }
    }

    fn pick_selection(&self, available_paths: &[CString]) -> Option<CString> {
        let persisted_serial = self.persistence.lock().unwrap().read_selected_serial();
        let states = self.orion_states.lock().unwrap();

        let mut candidates: Vec<(&CString, String)> = available_paths
            .iter()
            .filter_map(|path| {
                let orion_state = states.orion_states.get(path)?;

                (orion_state.inits_completed == OrionStateInitSteps::ALL).then(|| {
                    (
                        path,
                        String::from_utf8_lossy(&orion_state.serial).into_owned(),
                    )
                })
            })
            .collect();

        candidates.sort_by(|a, b| a.1.cmp(&b.1));

        candidates
            .iter()
            .find(|(_, serial)| Some(serial) == persisted_serial.as_ref())
            .or(candidates.first())
            .map(|(path, _)| (*path).clone())
    }

    fn persist_selection(&self, selected_hid_path: &CString) {
        let mut states = self.orion_states.lock().unwrap();

        let Some(orion_state) = states.orion_states.get_mut(selected_hid_path) else {
            return;
        };

        if orion_state.inits_completed != OrionStateInitSteps::ALL {
            return;
        }

        let persistence = self.persistence.lock().unwrap();
        let serial = String::from_utf8_lossy(&orion_state.serial).into_owned();

        let Some(labels) = persistence.read(&serial) else {
            return;
        };

        orion_state.labels = Arc::new(labels);

        persistence.write(orion_state);
    }

    pub(crate) fn poll_and_update_state(&mut self) {
        if !self.try_connect() {
            self.discard_pending_writes();

            thread::sleep(Self::DEVICE_REFRESH_INTERVAL);

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

        let state_exists = self
            .orion_states
            .lock()
            .unwrap()
            .orion_states
            .contains_key(selected_hid_path);

        if !state_exists {
            thread::sleep(Duration::from_millis(5));

            return;
        }

        let poll_result = match self.orion_hid_devices.get_mut(selected_hid_path) {
            Some(device) => {
                if device.ensure_init() {
                    device.read_packet()
                } else {
                    Err(())
                }
            }
            None => {
                thread::sleep(Duration::from_millis(5));

                return;
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
        let selected_hid_path = self.orion_states.lock().unwrap().selected_hid_path.clone();

        let selection_changed = selected_hid_path != self.last_selected_hid_path;

        let refresh_due = self
            .last_device_refresh
            .is_none_or(|last_refresh| last_refresh.elapsed() >= Self::DEVICE_REFRESH_INTERVAL);

        if !refresh_due && !selection_changed {
            return selected_hid_path
                .as_ref()
                .is_some_and(|path| self.orion_hid_devices.contains_key(path));
        }

        if let Err(error) = self.hid_api.refresh_devices() {
            eprintln!("Failed to refresh HID devices: {error}");
            self.last_device_refresh = Some(Instant::now());

            return selected_hid_path
                .as_ref()
                .is_some_and(|path| self.orion_hid_devices.contains_key(path));
        }

        self.last_device_refresh = Some(Instant::now());

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
                .is_some_and(|path| !available_paths.contains(path))
            {
                states.selected_hid_path = None;
            }

            states.selected_hid_path.clone()
        };

        self.last_selected_hid_path = selected_hid_path.clone();

        if selection_changed && let Some(path) = selected_hid_path.as_ref() {
            self.persist_selection(path);
        }

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

        if selected_hid_path.is_none()
            && let Some(path) = self.pick_selection(&available_paths)
        {
            self.orion_states.lock().unwrap().selected_hid_path = Some(path);

            return self.try_connect();
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
