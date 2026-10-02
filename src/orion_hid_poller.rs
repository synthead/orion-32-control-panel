use hidapi::HidApi;
use hidapi::HidDevice;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;

use crate::orion_state::OrionState;

pub struct OrionHidPoller {
    hid_api: Option<HidApi>,
    orion_hid: Option<HidDevice>,
    orion_state: Arc<Mutex<OrionState>>,
    hid_write_rx: mpsc::Receiver<[u8; 64]>,
}

impl OrionHidPoller {
    const USB_VENDOR: u16 = 0x23e5;
    const USB_PRODUCT: u16 = 0xa040;

    pub fn new(
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

    pub fn poll_and_update_state(&mut self) {
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

    pub fn update_orion_state(hid_data: &[u8; 64], orion_state: &mut OrionState) {
        if hid_data[0] == 0x73 {
            orion_state.left_vu_source = hid_data[30] & 0x0f;
            orion_state.right_vu_source = hid_data[30] >> 4;
            orion_state.clock_source = hid_data[19];
            orion_state.sample_rate = hid_data[18];
            orion_state.line_out = hid_data[28];

            let vu = if hid_data[31] == 1 {
                &mut orion_state.left_vu_values
            } else {
                &mut orion_state.right_vu_values
            };

            *vu = hid_data[32..].try_into().unwrap();
        }

        if hid_data[0] == 0x75 {
            orion_state.serial = hid_data[36..49].try_into().unwrap();
        }
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
