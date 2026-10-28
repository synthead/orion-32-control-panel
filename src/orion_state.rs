use crate::routing_constants::{RoutingColumns, RoutingLabelColors, RoutingLabels, RoutingRows};

pub(crate) struct OrionStateInitSteps;

impl OrionStateInitSteps {
    pub(crate) const SERIAL_NUMBER: u8 = 0b000001;
    pub(crate) const FIRMWARE_VERSION: u8 = 0b000010;
    pub(crate) const ROUTES: u8 = 0b000100;
    pub(crate) const MIXER_LEVELS: u8 = 0b001000;
    pub(crate) const MIXER_PANS_SOLOS_MUTES: u8 = 0b010000;
    pub(crate) const AD_PHASES: u8 = 0b100000;
}

#[derive(Clone)]
pub(crate) struct OrionState {
    pub(crate) labels: [[String; 32]; 7],

    pub(crate) source_counts: [usize; 7],
    pub(crate) target_counts: [usize; 7],

    pub(crate) inits_completed: u8,

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
        let labels = std::array::from_fn(|group| {
            let start_at = if group == 3 { 33 } else { 1 };

            std::array::from_fn(|i| (i + start_at).to_string())
        });

        Self {
            labels,

            source_counts: [0; 7],
            target_counts: [0; 7],

            inits_completed: 0,

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

impl OrionState {
    pub(crate) fn calculate_channel_counts(&mut self) {
        let usb_channel_count = if self.usb_channels_spdif_src & 0x02 == 0x00 {
            24
        } else {
            32
        };

        let madi_1_count = match self.sample_rate {
            5 | 6 => 16,
            _ => 32,
        };

        let madi_2_count = match self.sample_rate {
            0 | 1 | 2 => 32,
            _ => 0,
        };

        let adat_count = match self.sample_rate {
            3 | 4 => 8,
            5 | 6 => 4,
            _ => 16,
        };

        self.source_counts = [
            32,
            usb_channel_count.try_into().unwrap(),
            madi_1_count.try_into().unwrap(),
            madi_2_count.try_into().unwrap(),
            adat_count.try_into().unwrap(),
            2,
            2,
        ];

        self.target_counts = [
            32,
            usb_channel_count.try_into().unwrap(),
            madi_1_count.try_into().unwrap(),
            madi_2_count.try_into().unwrap(),
            adat_count.try_into().unwrap(),
            2,
            32,
        ];
    }

    pub(crate) fn column_row_for_route(&self, route: u8, truncate: bool) -> Option<[u8; 2]> {
        let column = route / RoutingColumns::STEP;
        let row = route % RoutingColumns::STEP;

        if truncate
            && row != RoutingRows::SPECIAL
            && column as usize >= self.source_counts[row as usize]
        {
            return None;
        }

        Some([column, row])
    }

    pub(crate) fn label_for_route(&self, route: u8, truncate: bool) -> String {
        if let Some([column, row]) = self.column_row_for_route(route, truncate) {
            if row == RoutingRows::SPECIAL {
                return match column {
                    0 => RoutingLabels::OSC_1.to_string(),
                    1 => RoutingLabels::OSC_2.to_string(),
                    _ => RoutingLabels::MUTE.to_string(),
                };
            }

            return self.labels[row as usize][column as usize].clone();
        }

        RoutingLabels::MUTE.to_string()
    }

    pub(crate) fn color_for_route(&self, route: u8, truncate: bool) -> egui::Color32 {
        if let Some([column, row]) = self.column_row_for_route(route, truncate) {
            if row == RoutingRows::SPECIAL {
                return match column {
                    0 => RoutingLabelColors::OSC_1,
                    1 => RoutingLabelColors::OSC_2,
                    _ => RoutingLabelColors::MUTE,
                };
            }

            return match row {
                RoutingRows::ADC_DAC => RoutingLabelColors::ADC_DAC,
                RoutingRows::USB => RoutingLabelColors::USB,
                RoutingRows::MADI_1 => RoutingLabelColors::MADI,
                RoutingRows::MADI_2 => RoutingLabelColors::MADI,
                RoutingRows::ADAT => RoutingLabelColors::ADAT,
                RoutingRows::SPDIF => RoutingLabelColors::SPDIF,
                RoutingRows::MIX_LR => RoutingLabelColors::MIX_LR,

                _ => egui::Color32::BLACK,
            };
        }

        RoutingLabelColors::MUTE
    }

    pub(crate) fn hex_color_for_route(&self, route: u8, truncate: bool) -> String {
        let egui_color = self.color_for_route(route, truncate);

        format!(
            "#{:02x}{:02x}{:02x}",
            egui_color.r(),
            egui_color.g(),
            egui_color.b()
        )
    }
}
