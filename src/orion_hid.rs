use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;

use crate::orion_state::OrionState;

pub(crate) struct OrionHid {
    hid_write_tx: mpsc::Sender<[u8; 64]>,
    orion_state: Arc<Mutex<OrionState>>,
}

impl OrionHid {
    pub(crate) fn new(
        hid_write_tx: mpsc::Sender<[u8; 64]>,
        orion_state: Arc<Mutex<OrionState>>,
    ) -> Self {
        Self {
            hid_write_tx,
            orion_state,
        }
    }

    pub(crate) fn queue_init(&mut self) {
        self.queue_get_serial();
        self.queue_get_routes();
        self.queue_get_mixer_levels();
        self.queue_get_mixer_pans();
        self.queue_get_ad_phases();
    }

    pub(crate) fn queue_get_serial(&mut self) {
        self.queue_packet(0x74, 0x10, 0x00, 0x01, &[]);
    }

    pub(crate) fn queue_get_routes(&mut self) {
        self.queue_packet(0x74, 0x10, 0x00, 0x02, &[]);
    }

    pub(crate) fn queue_get_mixer_levels(&mut self) {
        self.queue_packet(0x74, 0x10, 0x00, 0x04, &[]);
    }

    pub(crate) fn queue_get_mixer_pans(&mut self) {
        self.queue_packet(0x74, 0x10, 0x00, 0x05, &[]);
    }

    pub(crate) fn queue_get_ad_phases(&mut self) {
        self.queue_packet(0x74, 0x10, 0x00, 0x06, &[]);
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

        let mut orion_state = self.orion_state.lock().unwrap();
        orion_state.routes[target as usize] = *routes;
    }

    pub(crate) fn queue_mixer_channel(&mut self, channel: u8, level: u8, pan_solo_mute: u8) {
        self.queue_packet(
            0x70,
            0x14,
            0x00,
            0x00,
            &[0x90, channel, level, pan_solo_mute],
        );

        let mut orion_state = self.orion_state.lock().unwrap();
        orion_state.mixer_levels[channel as usize] = level;
        orion_state.mixer_pans_solos_mutes[channel as usize] = pan_solo_mute;
    }

    pub(crate) fn queue_ad_phase(&mut self, ad_phase: u8) {
        self.queue_packet(0x70, 0x12, 0x00, 0x00, &[0x11, ad_phase]);

        let mut orion_state = self.orion_state.lock().unwrap();

        let ad_phase_i = (ad_phase & 0x7f) as usize;
        let byte_i = ad_phase_i / 8;
        let bit_i = ad_phase_i % 8;

        if ad_phase & 0x80 != 0 {
            orion_state.ad_phases[byte_i] |= 1 << bit_i;
        } else {
            orion_state.ad_phases[byte_i] &= !(1 << bit_i);
        }
    }

    pub(crate) fn queue_preset(&mut self, preset: u8) {
        self.queue_packet(0x70, 0x12, 0x00, 0x00, &[0x06, preset]);
    }

    pub(crate) fn queue_preset_write(&mut self, preset: u8) {
        self.queue_packet(0x70, 0x12, 0x00, 0x00, &[0x05, preset]);
    }

    fn queue_packet(&mut self, byte_0: u8, byte_4: u8, byte_8: u8, byte_12: u8, payload: &[u8]) {
        let mut packet = [0u8; 64];

        packet[0] = byte_0;
        packet[4] = byte_4;
        packet[8] = byte_8;
        packet[12] = byte_12;
        packet[16..16 + payload.len()].copy_from_slice(payload);

        let _ = self.hid_write_tx.send(packet);
    }
}
