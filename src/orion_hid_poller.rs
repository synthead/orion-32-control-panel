use hidapi::{HidApi, HidDevice};
use std::collections::HashMap;
use std::ffi::CString;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

use crate::orion_hid::OrionHid;
use crate::orion_state::{OrionState, OrionStateInitSteps, OrionStates};

pub(crate) struct OrionHidDevice {
    hid_device: HidDevice,

    init_serial_firmware_written: bool,
    init_routes_mixer_ad_phases_written: bool,
}

impl OrionHidDevice {
    const READ_TIMEOUT_MS: i32 = 100;

    pub(crate) fn new(hid_device: HidDevice) -> Self {
        Self {
            hid_device,

            init_serial_firmware_written: false,
            init_routes_mixer_ad_phases_written: false,
        }
    }

    fn write_packet(&self, packet: &[u8; 64]) -> bool {
        let mut packet_with_report = [0u8; 65];
        packet_with_report[1..].copy_from_slice(packet);

        self.hid_device.write(&packet_with_report).is_ok()
    }

    fn ensure_init(&mut self) -> bool {
        if !self.init_serial_firmware_written {
            for packet in OrionHid::create_get_serial_firmware() {
                if !self.write_packet(&packet) {
                    return false;
                }
            }

            self.init_serial_firmware_written = true;
        }

        if !self.init_routes_mixer_ad_phases_written {
            for packet in OrionHid::create_get_routes_mixer_ad_phases() {
                if !self.write_packet(&packet) {
                    return false;
                }
            }

            self.init_routes_mixer_ad_phases_written = true;
        }

        true
    }

    fn poll(&mut self) -> Result<Option<[u8; 64]>, ()> {
        if !self.ensure_init() {
            return Err(());
        }

        let mut hid_data = [0u8; 64];

        match self
            .hid_device
            .read_timeout(&mut hid_data, Self::READ_TIMEOUT_MS)
        {
            Ok(64) => Ok(Some(hid_data)),
            Ok(_) => Ok(None),
            Err(_) => Err(()),
        }
    }

    fn update_orion_state(&mut self, hid_data: &[u8; 64], orion_state: &mut OrionState) {
        match (hid_data[0], hid_data[4], hid_data[12]) {
            (0x73, 0x00, 0x00) => {
                self.transform_vu(hid_data, orion_state);
            }
            (0x75, 0x10, 0x00) => {
                Self::transform_firmware_version(hid_data, orion_state);
            }
            (0x75, 0x10, 0x01) => {
                Self::transform_serial(hid_data, orion_state);
            }
            (0x75, 0x10, 0x02) => {
                Self::transform_routes(hid_data, orion_state);
            }
            (0x75, 0x10, 0x04) => {
                Self::transform_mixer_levels(hid_data, orion_state);
            }
            (0x75, 0x10, 0x05) => {
                Self::transform_mixer_pans_solos_mutes(hid_data, orion_state);
            }
            (0x75, 0x10, 0x06) => {
                Self::transform_ad_phases(hid_data, orion_state);
            }
            _ => {}
        }
    }

    fn transform_vu(&mut self, hid_data: &[u8; 64], orion_state: &mut OrionState) {
        let last_preset = orion_state.preset_standby & 0x07;

        orion_state.left_vu_source = hid_data[30] & 0x0f;
        orion_state.right_vu_source = hid_data[30] >> 4;

        orion_state.clock_source = hid_data[19];
        orion_state.sample_rate = hid_data[18];

        orion_state.line_out = hid_data[28];
        orion_state.signals = hid_data[17];
        orion_state.frequency = hid_data[21..24].try_into().unwrap();

        orion_state.preset_standby = hid_data[16];

        orion_state.trims = hid_data[24];
        orion_state.oscillators = hid_data[20];
        orion_state.brightness = hid_data[26];
        orion_state.madi_smux = hid_data[24];
        orion_state.usb_channels_spdif_src = hid_data[25];

        let vu = if hid_data[31] == 1 {
            &mut orion_state.left_vu_values
        } else {
            &mut orion_state.right_vu_values
        };

        *vu = hid_data[32..].try_into().unwrap();

        if last_preset != u8::MAX && last_preset != orion_state.preset_standby & 0x07 {
            self.init_routes_mixer_ad_phases_written = false;
        }
    }

    fn transform_serial(hid_data: &[u8; 64], orion_state: &mut OrionState) {
        orion_state.serial = hid_data[36..49].try_into().unwrap();
        orion_state.hardware_version = hid_data[52..60].try_into().unwrap();

        orion_state.inits_completed |= OrionStateInitSteps::SERIAL_NUMBER;
    }

    fn transform_firmware_version(hid_data: &[u8; 64], orion_state: &mut OrionState) {
        orion_state.firmware_version = hid_data[16..18].try_into().unwrap();

        orion_state.inits_completed |= OrionStateInitSteps::FIRMWARE_VERSION;
    }

    fn transform_routes(hid_data: &[u8; 64], orion_state: &mut OrionState) {
        let source_index = hid_data[16] as usize;

        orion_state.routes[source_index] = hid_data[17..49].try_into().unwrap();

        orion_state.inits_completed |= OrionStateInitSteps::ROUTES;
    }

    fn transform_mixer_levels(hid_data: &[u8; 64], orion_state: &mut OrionState) {
        orion_state.mixer_levels = hid_data[16..49].try_into().unwrap();

        orion_state.inits_completed |= OrionStateInitSteps::MIXER_LEVELS;
    }

    fn transform_mixer_pans_solos_mutes(hid_data: &[u8; 64], orion_state: &mut OrionState) {
        orion_state.mixer_pans_solos_mutes = hid_data[16..49].try_into().unwrap();

        orion_state.inits_completed |= OrionStateInitSteps::MIXER_PANS_SOLOS_MUTES;
    }

    fn transform_ad_phases(hid_data: &[u8; 64], orion_state: &mut OrionState) {
        orion_state.ad_phases = hid_data[17..21].try_into().unwrap();

        orion_state.inits_completed |= OrionStateInitSteps::AD_PHASES;
    }
}

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

        let selected_hid_path = self
            .orion_states
            .lock()
            .unwrap()
            .selected_hid_path
            .clone()
            .unwrap();

        while let Ok(packet) = self.hid_write_rx.try_recv() {
            let write_success = self
                .orion_hid_devices
                .get(&selected_hid_path)
                .is_some_and(|device| device.write_packet(&packet));

            if !write_success {
                self.disconnect();
                self.discard_pending_writes();
                return;
            }
        }

        let poll_result = match self.orion_hid_devices.get_mut(&selected_hid_path) {
            Some(device) => device.poll(),
            None => Err(()),
        };

        let hid_data = match poll_result {
            Ok(Some(hid_data)) => hid_data,
            Ok(None) => return,
            Err(()) => {
                self.disconnect();
                self.discard_pending_writes();
                return;
            }
        };

        let mut states = self.orion_states.lock().unwrap();

        if let (Some(device), Some(orion_state)) = (
            self.orion_hid_devices.get_mut(&selected_hid_path),
            states.orion_states.get_mut(&selected_hid_path),
        ) {
            device.update_orion_state(&hid_data, orion_state);
        }
    }

    fn try_connect(&mut self) -> bool {
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

        self.orion_states
            .lock()
            .unwrap()
            .selected_hid_path
            .is_some()
    }

    fn disconnect(&mut self) {
        let mut states = self.orion_states.lock().unwrap();

        if let Some(path) = states.selected_hid_path.take() {
            self.orion_hid_devices.remove(&path);
            states.orion_states.remove(&path);
        }

        states.selected_hid_path = self.orion_hid_devices.keys().next().cloned();
    }

    fn discard_pending_writes(&self) {
        while self.hid_write_rx.try_recv().is_ok() {}
    }
}
