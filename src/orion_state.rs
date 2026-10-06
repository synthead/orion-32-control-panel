#[derive(Clone, Default)]
pub(crate) struct OrionState {
    pub(crate) serial: [u8; 13],
    pub(crate) left_vu_source: u8,
    pub(crate) left_vu_values: [u8; 32],
    pub(crate) right_vu_source: u8,
    pub(crate) right_vu_values: [u8; 32],
    pub(crate) clock_source: u8,
    pub(crate) sample_rate: u8,
    pub(crate) line_out: u8,
    pub(crate) signals: u8,

    pub(crate) routes: [[u8; 32]; 7],
}
