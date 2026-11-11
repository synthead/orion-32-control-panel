use crate::constants::{RoutingColumns, RoutingLabelColors, RoutingLabels, RoutingRows};
use std::collections::HashMap;
use std::ffi::CString;
use std::sync::Arc;

pub(crate) struct OrionStateInitSteps;

impl OrionStateInitSteps {
    pub(crate) const SERIAL_NUMBER: u8 = 0x01;
    pub(crate) const FIRMWARE_VERSION: u8 = 0x02;
    pub(crate) const ACTIVATE: u8 = 0x04;
    pub(crate) const ROUTES: u8 = 0x08;
    pub(crate) const MIXER_LEVELS: u8 = 0x10;
    pub(crate) const MIXER_PANS_SOLOS_MUTES: u8 = 0x20;
    pub(crate) const AD_PHASES: u8 = 0x40;
    pub(crate) const VU: u8 = 0x80;
    pub(crate) const ALL: u8 = 0xff;
}

pub(crate) struct OrionStateSignals;

impl OrionStateSignals {
    pub(crate) const LOCK: u8 = 0x02;
    pub(crate) const ATOMIC: u8 = 0x04;
    pub(crate) const USB: u8 = 0x08;
    pub(crate) const MADI: u8 = 0x10;
    pub(crate) const ADAT_1: u8 = 0x20;
    pub(crate) const SPDIF: u8 = 0x40;
    pub(crate) const ADAT_2: u8 = 0x80;
}

#[derive(Default)]
pub(crate) struct OrionStates {
    pub(crate) selected_hid_path: Option<CString>,
    pub(crate) orion_states: HashMap<CString, OrionState>,
}

impl OrionStates {
    pub(crate) fn current(&self) -> Option<&OrionState> {
        self.selected_hid_path
            .as_ref()
            .and_then(|hid_path| self.orion_states.get(hid_path))
    }

    pub(crate) fn current_mut(&mut self) -> Option<&mut OrionState> {
        self.selected_hid_path
            .as_ref()
            .and_then(|path| self.orion_states.get_mut(path))
    }
}

#[derive(Clone)]
pub(crate) struct OrionState {
    pub(crate) labels: Arc<[[String; 32]; 7]>,

    pub(crate) source_counts: [usize; 7],
    pub(crate) target_counts: [usize; 7],

    pub(crate) inits_completed: u8,

    pub(crate) serial: [u8; 13],
    pub(crate) hardware_version: [u8; 8],
    pub(crate) firmware_version: [u8; 2],
    pub(crate) activate: u8,

    pub(crate) left_vu_source: u8,
    pub(crate) left_vu_values: [u8; 32],
    pub(crate) right_vu_source: u8,
    pub(crate) right_vu_values: [u8; 32],
    pub(crate) clock_source: u8,
    pub(crate) sample_rate: u8,
    pub(crate) line_out: u8,
    pub(crate) signals: u8,
    pub(crate) frequency: [u8; 3],
    pub(crate) preset_standby: u8,

    pub(crate) madi_channels: u8,
    pub(crate) adat_channels: u8,

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
        let labels =
            RoutingLabels::DEFAULT_LABEL_NUMBERS.map(|row| row.map(|number| number.to_string()));

        Self {
            labels: Arc::new(labels),

            source_counts: [0; 7],
            target_counts: [0; 7],

            inits_completed: 0,

            serial: [0; 13],
            hardware_version: [0; 8],
            firmware_version: [0; 2],
            activate: 0,

            left_vu_source: 0,
            left_vu_values: [0; 32],
            right_vu_source: 0,
            right_vu_values: [0; 32],
            clock_source: 0,
            sample_rate: 0,
            line_out: 0,
            signals: 0,
            frequency: [0; 3],
            preset_standby: u8::MAX,

            madi_channels: 0,
            adat_channels: 0,

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

        let madi_1_count = self.madi_channels.clamp(0, 32) as usize;
        let madi_2_count = self.madi_channels.saturating_sub(32).clamp(0, 32) as usize;
        let adat_count = self.adat_channels as usize;

        self.source_counts = [
            32,
            usb_channel_count,
            madi_1_count,
            madi_2_count,
            adat_count,
            2,
            2,
        ];

        self.target_counts = [
            32,
            usb_channel_count,
            madi_1_count,
            madi_2_count,
            adat_count,
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

    pub(crate) fn label_for_route(&self, route: u8, truncate: bool) -> &str {
        if let Some([column, row]) = self.column_row_for_route(route, truncate) {
            if row == RoutingRows::SPECIAL {
                return match column {
                    0 => RoutingLabels::OSC_1,
                    1 => RoutingLabels::OSC_2,
                    _ => RoutingLabels::MUTE,
                };
            }

            return &self.labels[row as usize][column as usize];
        }

        RoutingLabels::MUTE
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
