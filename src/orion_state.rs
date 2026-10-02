#[derive(Default)]
pub struct OrionState {
    pub serial: [u8; 13],
    pub left_vu_source: u8,
    pub left_vu_values: [u8; 32],
    pub right_vu_source: u8,
    pub right_vu_values: [u8; 32],
    pub clock_source: u8,
    pub sample_rate: u8,
    pub line_out: u8,
}
