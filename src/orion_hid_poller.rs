use hidapi::{HidApi, HidDevice};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

use crate::orion_state::OrionState;

pub(crate) struct OrionHidPoller {
    hid_api: Option<HidApi>,
    hid_device: Option<HidDevice>,
    orion_state: Arc<Mutex<OrionState>>,
    hid_write_rx: mpsc::Receiver<[u8; 64]>,
}

impl OrionHidPoller {
    const USB_VENDOR: u16 = 0x23e5;
    const USB_PRODUCT: u16 = 0xa040;
    const READ_TIMEOUT_MS: i32 = 100;

    pub(crate) fn new(
        orion_state: Arc<Mutex<OrionState>>,
        hid_write_rx: mpsc::Receiver<[u8; 64]>,
    ) -> Self {
        Self {
            orion_state,
            hid_write_rx,
            hid_api: None,
            hid_device: None,
        }
    }

    pub(crate) fn poll_and_update_state(&mut self) {
        if self.hid_device.is_none() && !self.try_connect() {
            self.discard_pending_writes();
            return;
        }

        while let Ok(packet) = self.hid_write_rx.try_recv() {
            let result = self.hid_device.as_ref().unwrap().write(&packet);

            if result.is_err() {
                self.disconnect();
                self.discard_pending_writes();
                return;
            }
        }

        let mut hid_data = [0u8; 64];

        match self
            .hid_device
            .as_ref()
            .unwrap()
            .read_timeout(&mut hid_data, Self::READ_TIMEOUT_MS)
        {
            Ok(64) => {
                let mut orion_state = self.orion_state.lock().unwrap();

                Self::update_orion_state(&hid_data, &mut orion_state);
            }
            Ok(_) => {}
            Err(_) => {
                self.disconnect();
            }
        }
    }

    fn update_orion_state(hid_data: &[u8; 64], orion_state: &mut OrionState) {
        match (hid_data[0], hid_data[4], hid_data[12]) {
            (0x73, 0x00, 0x00) => {
                Self::transform_vu(hid_data, orion_state);
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

    fn transform_vu(hid_data: &[u8; 64], orion_state: &mut OrionState) {
        orion_state.left_vu_source = hid_data[30] & 0x0f;
        orion_state.right_vu_source = hid_data[30] >> 4;

        orion_state.clock_source = hid_data[19];
        orion_state.sample_rate = hid_data[18];

        orion_state.line_out = hid_data[28];
        orion_state.signals = hid_data[17];

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
    }

    fn transform_serial(hid_data: &[u8; 64], orion_state: &mut OrionState) {
        orion_state.serial = hid_data[36..49].try_into().unwrap();
        orion_state.hardware_version = hid_data[52..60].try_into().unwrap();
    }

    fn transform_firmware_version(hid_data: &[u8; 64], orion_state: &mut OrionState) {
        orion_state.firmware_version = hid_data[16..18].try_into().unwrap();
    }

    fn transform_routes(hid_data: &[u8; 64], orion_state: &mut OrionState) {
        let source_index = hid_data[16] as usize;

        orion_state.routes[source_index] = hid_data[17..49].try_into().unwrap();
    }

    fn transform_mixer_levels(hid_data: &[u8; 64], orion_state: &mut OrionState) {
        orion_state.mixer_levels = hid_data[16..49].try_into().unwrap();
    }

    fn transform_mixer_pans_solos_mutes(hid_data: &[u8; 64], orion_state: &mut OrionState) {
        orion_state.mixer_pans_solos_mutes = hid_data[16..49].try_into().unwrap();
    }

    fn transform_ad_phases(hid_data: &[u8; 64], orion_state: &mut OrionState) {
        orion_state.ad_phases = hid_data[17..21].try_into().unwrap();
    }

    fn try_connect(&mut self) -> bool {
        let hid_api = match HidApi::new() {
            Ok(hid_api) => hid_api,
            Err(_) => {
                self.set_present(false);
                return false;
            }
        };

        let hid_device = match hid_api.open(Self::USB_VENDOR, Self::USB_PRODUCT) {
            Ok(hid_device) => hid_device,
            Err(_) => {
                self.set_present(false);
                return false;
            }
        };

        self.hid_api = Some(hid_api);
        self.hid_device = Some(hid_device);

        self.set_present(true);

        true
    }

    fn disconnect(&mut self) {
        self.hid_device = None;
        self.hid_api = None;

        self.set_present(false);
    }

    fn set_present(&self, present: bool) {
        let mut orion_state = self.orion_state.lock().unwrap();

        orion_state.present = present;

        if !present {
          orion_state.initialized = false;
        }
    }

    fn discard_pending_writes(&self) {
        while self.hid_write_rx.try_recv().is_ok() {}
    }
}
