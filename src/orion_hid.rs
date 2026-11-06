use std::sync::{Arc, Mutex, mpsc};

use crate::orion_persistence::OrionPersistence;
use crate::orion_state::OrionStates;

pub(crate) struct OrionHid {
    hid_write_tx: mpsc::Sender<[u8; 64]>,
    orion_states: Arc<Mutex<OrionStates>>,
    persistence: Arc<Mutex<OrionPersistence>>,
}

impl OrionHid {
    pub(crate) fn new(
        hid_write_tx: mpsc::Sender<[u8; 64]>,
        orion_states: Arc<Mutex<OrionStates>>,
        persistence: Arc<Mutex<OrionPersistence>>,
    ) -> Self {
        Self {
            hid_write_tx,
            orion_states,
            persistence,
        }
    }

    pub(crate) fn queue_line_out(&mut self, line_out: u8) {
        self.queue_packet(0x70, 0x12, 0x00, 0x00, &[0x08, line_out]);
    }

    pub(crate) fn queue_vu_source_left(&mut self, vu_source_left: u8) {
        self.queue_packet(0x70, 0x13, 0x00, 0x00, &[0x49, 0x00, vu_source_left]);
    }

    pub(crate) fn queue_vu_source_right(&mut self, vu_source_right: u8) {
        self.queue_packet(0x70, 0x13, 0x00, 0x00, &[0x49, 0x01, vu_source_right]);
    }

    pub(crate) fn queue_clock_source(&mut self, clock_source: u8) {
        self.queue_packet(0x70, 0x12, 0x00, 0x00, &[0x03, clock_source]);
    }

    pub(crate) fn queue_sample_rate(&mut self, sample_rate: u8) {
        self.queue_packet(0x70, 0x16, 0x00, 0x00, &[0xc2, 0x04, sample_rate]);
    }

    pub(crate) fn queue_routes(&mut self, target: u8, routes: &[u8; 32]) {
        let mut routes_payload = [0u8; 35];

        routes_payload[..3].copy_from_slice(&[0xc1, 0x21, target]);
        routes_payload[3..].copy_from_slice(routes);

        self.queue_packet(0x70, 0x33, 0x00, 0x00, &routes_payload);

        let mut states = self.orion_states.lock().unwrap();

        if let Some(selected_hid_path) = states.selected_hid_path.clone()
            && let Some(orion_state) = states.orion_states.get_mut(&selected_hid_path)
        {
            orion_state.routes[target as usize] = *routes;
        }
    }

    pub(crate) fn queue_mixer_channel(&mut self, channel: u8, level: u8, pan_solo_mute: u8) {
        self.queue_packet(
            0x70,
            0x14,
            0x00,
            0x00,
            &[0x90, channel, level, pan_solo_mute],
        );

        let mut states = self.orion_states.lock().unwrap();

        if let Some(selected_hid_path) = states.selected_hid_path.clone()
            && let Some(orion_state) = states.orion_states.get_mut(&selected_hid_path)
        {
            orion_state.mixer_levels[channel as usize] = level;
            orion_state.mixer_pans_solos_mutes[channel as usize] = pan_solo_mute;
        }
    }

    pub(crate) fn queue_ad_phase(&mut self, ad_phase: u8) {
        self.queue_packet(0x70, 0x12, 0x00, 0x00, &[0x11, ad_phase]);

        let mut states = self.orion_states.lock().unwrap();

        if let Some(selected_hid_path) = states.selected_hid_path.clone()
            && let Some(orion_state) = states.orion_states.get_mut(&selected_hid_path)
        {
            let ad_phase_i = (ad_phase & 0x7f) as usize;
            let byte_i = ad_phase_i / 8;
            let bit_i = ad_phase_i % 8;

            if ad_phase & 0x80 != 0 {
                orion_state.ad_phases[byte_i] |= 1 << bit_i;
            } else {
                orion_state.ad_phases[byte_i] &= !(1 << bit_i);
            }
        }
    }

    pub(crate) fn queue_preset(&mut self, preset: u8) {
        self.queue_packet(0x70, 0x12, 0x00, 0x00, &[0x06, preset]);
    }

    pub(crate) fn queue_preset_write(&mut self, preset: u8) {
        {
            let orion_states = self.orion_states.lock().unwrap();

            if let Some(orion_state) = orion_states.current() {
                self.persistence.lock().unwrap().write(orion_state, preset);
            }
        }

        self.queue_packet(0x70, 0x12, 0x00, 0x00, &[0x05, preset]);
    }

    pub(crate) fn queue_standby(&mut self, standby: u8) {
        self.queue_packet(0x66, 0x10, standby, 0x00, &[]);
    }

    pub(crate) fn queue_trim(&mut self, trim: u8) {
        self.queue_packet(0x70, 0x12, 0x00, 0x00, &[0x0c, trim]);
    }

    pub(crate) fn queue_oscillators(&mut self, oscillators: u8) {
        self.queue_packet(0x70, 0x12, 0x00, 0x00, &[0x0a, oscillators]);
    }

    pub(crate) fn queue_brightness(&mut self, brightness: u8) {
        self.queue_packet(0x70, 0x13, 0x00, 0x00, &[0x44, brightness]);
    }

    pub(crate) fn queue_usb_channels(&mut self, usb_channels: u8) {
        self.queue_packet(0x70, 0x12, 0x00, 0x00, &[0x0e, usb_channels]);
    }

    pub(crate) fn queue_madi_smux(&mut self, madi_smux: u8) {
        self.queue_packet(0x70, 0x12, 0x00, 0x00, &[0x0b, madi_smux]);
    }

    pub(crate) fn queue_spdif_src(&mut self, spdif_src: u8) {
        self.queue_packet(0x70, 0x12, 0x00, 0x00, &[0x0d, spdif_src]);
    }

    fn queue_packet(&mut self, byte_0: u8, byte_4: u8, byte_8: u8, byte_12: u8, payload: &[u8]) {
        let packet = Self::create_packet(byte_0, byte_4, byte_8, byte_12, payload);

        let _ = self.hid_write_tx.send(packet);
    }

    pub(crate) fn create_get_serial_firmware() -> [[u8; 64]; 2] {
        [
            Self::create_get_serial(),
            Self::create_get_firmware_version(),
        ]
    }

    pub(crate) fn create_get_routes_mixer_ad_phases() -> [[u8; 64]; 4] {
        [
            Self::create_get_routes(),
            Self::create_get_mixer_levels(),
            Self::create_get_mixer_pans(),
            Self::create_get_ad_phases(),
        ]
    }

    pub(crate) fn create_get_serial() -> [u8; 64] {
        Self::create_packet(0x74, 0x10, 0x00, 0x01, &[])
    }

    pub(crate) fn create_get_firmware_version() -> [u8; 64] {
        Self::create_packet(0x74, 0x10, 0x00, 0x00, &[])
    }

    pub(crate) fn create_get_routes() -> [u8; 64] {
        Self::create_packet(0x74, 0x10, 0x00, 0x02, &[])
    }

    pub(crate) fn create_get_mixer_levels() -> [u8; 64] {
        Self::create_packet(0x74, 0x10, 0x00, 0x04, &[])
    }

    pub(crate) fn create_get_mixer_pans() -> [u8; 64] {
        Self::create_packet(0x74, 0x10, 0x00, 0x05, &[])
    }

    pub(crate) fn create_get_ad_phases() -> [u8; 64] {
        Self::create_packet(0x74, 0x10, 0x00, 0x06, &[])
    }

    fn create_packet(byte_0: u8, byte_4: u8, byte_8: u8, byte_12: u8, payload: &[u8]) -> [u8; 64] {
        let mut packet = [0u8; 64];

        packet[0] = byte_0;
        packet[4] = byte_4;
        packet[8] = byte_8;
        packet[12] = byte_12;
        packet[16..16 + payload.len()].copy_from_slice(payload);

        packet
    }
}
