use hidapi::HidApi;
use hidapi::HidDevice;

use crate::orion_state::OrionState;

#[derive(Default)]
pub struct OrionHid {
    hid_api: Option<HidApi>,
    orion_hid: Option<HidDevice>,
}

impl OrionHid {
    const USB_VENDOR: u16 = 0x23e5;
    const USB_PRODUCT: u16 = 0xa040;

    pub fn write_init(&mut self) {
        let packet_init = [
            0x74, 0x00, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];

        self.orion_hid()
            .write(&packet_init)
            .expect("Failed to write HID packet");
    }

    pub fn read_packet(&mut self) -> [u8; 64] {
        let mut hid_data = [0u8; 64];

        self.orion_hid()
            .read(&mut hid_data)
            .expect("Failed to read HID packet");

        hid_data
    }

    pub fn update_orion_state(hid_data: &[u8; 64], orion_state: &mut OrionState) -> bool {
        if !matches!(hid_data[0], 0x73 | 0x75) {
            return false;
        }

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

        true
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
