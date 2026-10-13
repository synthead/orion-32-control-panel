use hidapi::HidApi;
use hidapi::HidDevice;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;

use crate::orion_state::OrionState;

pub(crate) struct OrionHidPoller {
    hid_api: Option<HidApi>,
    orion_hid: Option<HidDevice>,
    orion_state: Arc<Mutex<OrionState>>,
    hid_write_rx: mpsc::Receiver<[u8; 64]>,
}

impl OrionHidPoller {
    const USB_VENDOR: u16 = 0x23e5;
    const USB_PRODUCT: u16 = 0xa040;

    pub(crate) fn new(
        orion_state: Arc<Mutex<OrionState>>,
        hid_write_rx: mpsc::Receiver<[u8; 64]>,
    ) -> Self {
        Self {
            orion_state,
            hid_write_rx,
            hid_api: None,
            orion_hid: None,
        }
    }

    pub(crate) fn poll_and_update_state(&mut self) {
        while let Ok(packet) = self.hid_write_rx.try_recv() {
            self.orion_hid()
                .write(&packet)
                .expect("Failed to write HID packet");
        }

        let mut hid_data = [0u8; 64];

        self.orion_hid()
            .read(&mut hid_data)
            .expect("Failed to read HID packet");

        let mut state = self.orion_state.lock().unwrap();
        Self::update_orion_state(&hid_data, &mut state);
    }

    fn update_orion_state(hid_data: &[u8; 64], orion_state: &mut OrionState) {
        match (hid_data[0], hid_data[4], hid_data[12]) {
            (0x73, 0x00, 0x00) => {
                Self::transform_vu(hid_data, orion_state);
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

        let vu = if hid_data[31] == 1 {
            &mut orion_state.left_vu_values
        } else {
            &mut orion_state.right_vu_values
        };

        *vu = hid_data[32..].try_into().unwrap();
    }

    fn transform_serial(hid_data: &[u8; 64], orion_state: &mut OrionState) {
        orion_state.serial = hid_data[36..49].try_into().unwrap();
    }

    fn transform_routes(hid_data: &[u8; 64], orion_state: &mut OrionState) {
        let source_index = hid_data[16] as usize;

        orion_state.routes[source_index] = hid_data[17..49].try_into().unwrap()
    }

    fn transform_mixer_levels(hid_data: &[u8; 64], orion_state: &mut OrionState) {
        orion_state.mixer_levels = hid_data[16..49].try_into().unwrap()
    }

    fn transform_mixer_pans_solos_mutes(hid_data: &[u8; 64], orion_state: &mut OrionState) {
        orion_state.mixer_pans_solos_mutes = hid_data[16..49].try_into().unwrap()
    }

    fn transform_ad_phases(hid_data: &[u8; 64], orion_state: &mut OrionState) {
        orion_state.ad_phases = hid_data[17..21].try_into().unwrap()
    }

    fn hid_api(&mut self) -> &mut HidApi {
        self.hid_api
            .get_or_insert_with(|| HidApi::new().expect("Failed to create API instance"))
    }

    fn orion_hid(&mut self) -> &mut HidDevice {
        if self.orion_hid.is_none() {
            let device = self
                .hid_api()
                .open(Self::USB_VENDOR, Self::USB_PRODUCT)
                .expect("Failed to open device");

            self.orion_hid = Some(device);
        }

        self.orion_hid.as_mut().unwrap()
    }
}
