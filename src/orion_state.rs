#[derive(Clone)]
pub(crate) struct OrionState {
    pub(crate) present: bool,
    pub(crate) acquired_serial_firmware: bool,
    pub(crate) acquired_routes_mixer_ad_phases: bool,

    pub(crate) serial: [u8; 13],
    pub(crate) hardware_version: [u8; 8],
    pub(crate) firmware_version: [u8; 2],

    pub(crate) left_vu_source: u8,
    pub(crate) left_vu_values: [u8; 32],
    pub(crate) right_vu_source: u8,
    pub(crate) right_vu_values: [u8; 32],
    pub(crate) clock_source: u8,
    pub(crate) sample_rate: u8,
    pub(crate) line_out: u8,
    pub(crate) signals: u8,
    pub(crate) preset_standby: u8,

    pub(crate) trims: u8,
    pub(crate) oscillators: u8,
    pub(crate) brightness: u8,
    pub(crate) madi_smux: u8,
    pub(crate) usb_channels_spdif_src: u8,

    pub(crate) routes: [[u8; 32]; 7],
    pub(crate) mixer_levels: [u8; 33],
    pub(crate) mixer_pans_solos_mutes: [u8; 33],
    pub(crate) ad_phases: [u8; 4],
}

impl Default for OrionState {
    fn default() -> Self {
        Self {
            present: false,
            acquired_serial_firmware: false,
            acquired_routes_mixer_ad_phases: false,

            serial: [0; 13],
            hardware_version: [0; 8],
            firmware_version: [0; 2],

            left_vu_source: 0,
            left_vu_values: [0; 32],
            right_vu_source: 0,
            right_vu_values: [0; 32],
            clock_source: 0,
            sample_rate: 0,
            line_out: 0,
            signals: 0,
            preset_standby: u8::MAX,

            trims: 0,
            oscillators: 0,
            brightness: 0,
            madi_smux: 0,
            usb_channels_spdif_src: 0,

            routes: [[0; 32]; 7],
            mixer_levels: [0x00; 33],
            mixer_pans_solos_mutes: [0x20; 33],
            ad_phases: [0x00; 4],
        }
    }
}
