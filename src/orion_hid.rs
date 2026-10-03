use std::sync::mpsc;

pub struct OrionHid {
    hid_write_tx: mpsc::Sender<[u8; 64]>,
}

impl OrionHid {
    pub fn new(hid_write_tx: mpsc::Sender<[u8; 64]>) -> Self {
        Self { hid_write_tx }
    }

    pub fn queue_init(&mut self) {
        self.queue_packet(0x74, 0x10, 0x01, &[]);
        self.queue_packet(0x74, 0x10, 0x02, &[]);
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

    fn queue_packet(&mut self, byte_0: u8, byte_4: u8, byte_12: u8, payload: &[u8]) {
        let mut packet = [0u8; 64];

        packet[0] = byte_0;
        packet[4] = byte_4;
        packet[12] = byte_12;
        packet[16..16 + payload.len()].copy_from_slice(payload);

        let _ = self.hid_write_tx.send(packet);
    }
}
