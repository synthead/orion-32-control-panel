use hidapi::HidDevice;

use crate::orion_hid::OrionHid;
use crate::orion_state::{OrionState, OrionStateInitSteps};

pub(crate) struct OrionHidDevice {
    hid_device: HidDevice,
}

impl OrionHidDevice {
    const READ_TIMEOUT_MS: i32 = 100;

    pub(crate) fn new(hid_device: HidDevice) -> Self {
        Self { hid_device }
    }

    pub(crate) fn write_packet(&self, packet: &[u8; 64]) -> bool {
        let mut packet_with_report = [0u8; 65];
        packet_with_report[1..].copy_from_slice(packet);

        self.hid_device.write(&packet_with_report).is_ok()
    }

    fn ensure_init(&mut self, orion_state: &OrionState) -> bool {
        let completed = orion_state.inits_completed;

        let serial_firmware_complete = completed & OrionStateInitSteps::SERIAL_NUMBER != 0
            && completed & OrionStateInitSteps::FIRMWARE_VERSION != 0;

        if !serial_firmware_complete {
            for packet in OrionHid::create_get_serial_firmware() {
                if !self.write_packet(&packet) {
                    return false;
                }
            }
        }

        let routes_mixer_ad_phases_complete = completed & OrionStateInitSteps::ROUTES != 0
            && completed & OrionStateInitSteps::MIXER_LEVELS != 0
            && completed & OrionStateInitSteps::MIXER_PANS_SOLOS_MUTES != 0
            && completed & OrionStateInitSteps::AD_PHASES != 0;

        if !routes_mixer_ad_phases_complete {
            for packet in OrionHid::create_get_routes_mixer_ad_phases() {
                if !self.write_packet(&packet) {
                    return false;
                }
            }
        }

        true
    }

    pub(crate) fn poll(&mut self, orion_state: &OrionState) -> Result<Option<[u8; 64]>, ()> {
        if !self.ensure_init(orion_state) {
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

    pub(crate) fn update_orion_state(&mut self, hid_data: &[u8; 64], orion_state: &mut OrionState) {
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
            orion_state.inits_completed &= !OrionStateInitSteps::ROUTES;
            orion_state.inits_completed &= !OrionStateInitSteps::MIXER_LEVELS;
            orion_state.inits_completed &= !OrionStateInitSteps::MIXER_PANS_SOLOS_MUTES;
            orion_state.inits_completed &= !OrionStateInitSteps::AD_PHASES;
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
