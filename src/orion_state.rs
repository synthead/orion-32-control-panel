#[derive(Clone)]
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
    pub(crate) preset_standby: u8,

    pub(crate) routes: [[u8; 32]; 7],
    pub(crate) mixer_levels: [u8; 33],
    pub(crate) mixer_pans_solos_mutes: [u8; 33],
    pub(crate) ad_phases: [u8; 4],
}

impl Default for OrionState {
    fn default() -> Self {
        Self {
            serial: [0; 13],
            left_vu_source: 0,
            left_vu_values: [0; 32],
            right_vu_source: 0,
            right_vu_values: [0; 32],
            clock_source: 0,
            sample_rate: 0,
            line_out: 0,
            signals: 0,
            preset_standby: 0,

            routes: [[0; 32]; 7],
            mixer_levels: [0x00; 33],
            mixer_pans_solos_mutes: [0x20; 33],
            ad_phases: [0x00; 4],
        }
    }
}
