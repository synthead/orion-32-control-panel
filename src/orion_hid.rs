use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;

use crate::orion_state::OrionState;

pub struct OrionHid {
    hid_write_tx: mpsc::Sender<[u8; 64]>,
    orion_state: Arc<Mutex<OrionState>>,
}

impl OrionHid {
    pub fn new(hid_write_tx: mpsc::Sender<[u8; 64]>, orion_state: Arc<Mutex<OrionState>>) -> Self {
        Self {
            hid_write_tx,
            orion_state,
        }
    }

    pub fn queue_init(&mut self) {
        self.queue_get_serial();
        self.queue_get_routes();
        self.queue_get_mixer_levels();
        self.queue_get_mixer_pans();
    }

    pub fn queue_get_serial(&mut self) {
        self.queue_packet(0x74, 0x10, 0x01, &[]);
    }

    pub fn queue_get_routes(&mut self) {
        self.queue_packet(0x74, 0x10, 0x02, &[]);
    }

    pub fn queue_get_mixer_levels(&mut self) {
        self.queue_packet(0x74, 0x10, 0x04, &[]);
    }

    pub fn queue_get_mixer_pans(&mut self) {
        self.queue_packet(0x74, 0x10, 0x05, &[]);
    }

    pub fn queue_line_out_volume(&mut self, line_out_volume: u8) {
        self.queue_packet(0x70, 0x12, 0x00, &[0x08, line_out_volume]);
    }

    pub fn queue_vu_source_left(&mut self, vu_source_left: u8) {
        self.queue_packet(0x70, 0x13, 0x00, &[0x49, 0x00, vu_source_left]);
    }

    pub fn queue_vu_source_right(&mut self, vu_source_right: u8) {
        self.queue_packet(0x70, 0x13, 0x00, &[0x49, 0x01, vu_source_right]);
    }

    pub fn queue_clock_source(&mut self, clock_source: u8) {
        self.queue_packet(0x70, 0x12, 0x00, &[0x03, clock_source]);
    }

    pub fn queue_sample_rate(&mut self, sample_rate: u8) {
        self.queue_packet(0x70, 0x16, 0x00, &[0xc2, 0x04, sample_rate]);
    }

    pub fn queue_routes(&mut self, target: u8, routes: &[u8; 32]) {
        let mut routes_payload = [0u8; 35];

        routes_payload[..3].copy_from_slice(&[0xc1, 0x21, target]);
        routes_payload[3..].copy_from_slice(routes);

        self.queue_packet(0x70, 0x33, 0x00, &routes_payload);

        let mut orion_state = self.orion_state.lock().unwrap();
        orion_state.routes[target as usize] = *routes;
    }

    pub fn queue_mixer_channel(&mut self, channel: u8, level: u8, pan_solo_mute: u8) {
        self.queue_packet(0x70, 0x14, 0x00, &[0x90, channel, level, pan_solo_mute]);

        let mut orion_state = self.orion_state.lock().unwrap();
        orion_state.mixer_levels[channel as usize] = level;
        orion_state.mixer_pans_solos_mutes[channel as usize] = pan_solo_mute;
    }

    fn queue_packet(&mut self, byte_0: u8, byte_4: u8, byte_12: u8, payload: &[u8]) {
        let mut packet = [0u8; 64];

        packet[0] = byte_0;
        packet[4] = byte_4;
        packet[12] = byte_12;
        packet[16..16 + payload.len()].copy_from_slice(payload);

        let _ = self.hid_write_tx.send(packet);
    }
}
