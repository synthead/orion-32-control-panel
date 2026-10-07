use std::sync::Arc;
use std::sync::Mutex;

use crate::orion_hid::OrionHid;
use crate::orion_state::OrionState;

// use crate::control_panel_helpers::ControlPanelHelpers;

use crate::render_asset;

struct ControlPanelMixerColors;

impl ControlPanelMixerColors {
    const LABEL: egui::Color32 = egui::Color32::from_rgb(0x20, 0xa4, 0xf7);
    const LABEL_TEXT: egui::Color32 = egui::Color32::WHITE;
    const VALUE_TEXT: egui::Color32 = egui::Color32::from_rgb(0x95, 0x95, 0x95);
}

struct ControlPanelMixerDimensions;

impl ControlPanelMixerDimensions {
    const STEP_X: f32 = 35.0;
    const FONT_SIZE_LABEL: f32 = 10.0;
    const FONT_SIZE_VALUE: f32 = 12.0;

    const LABEL_X: f32 = 5.0;
    const LABEL_Y: f32 = 239.0;
    const LABEL_WIDTH: f32 = 33.0;
    const LABEL_HEIGHT: f32 = 13.0;

    const PAN_KNOB_X: u16 = 5;
    const PAN_KNOB_Y: u16 = 283;
    const PAN_RAD: f32 = 2.35;

    const SLIDER_START_X: u16 = 11;
    const SLIDER_START_Y: u16 = 366;
    const SLIDER_DEPTH_Y: u16 = 178;

    const SOLO_MUTE_X: u16 = 7;
    const SOLO_Y: u16 = 604;
    const MUTE_Y: u16 = 622;

    const TEXT_X: f32 = 21.0;
    const PAN_TEXT_Y: f32 = 270.0;
    const LEVEL_TEXT_Y: f32 = 348.0;

    const MASTER_TEXT_X: f32 = 1150.0;
    const MASTER_TEXT_Y: f32 = 342.0;
}

pub(crate) struct ControlPanelMixer {
    orion_state: Arc<Mutex<OrionState>>,
    orion_hid: Arc<Mutex<OrionHid>>,
}

impl ControlPanelMixer {
    pub(crate) fn new(
        orion_state: Arc<Mutex<OrionState>>,
        orion_hid: Arc<Mutex<OrionHid>>,
    ) -> Self {
        Self {
            orion_state,
            orion_hid,
        }
    }

    pub(crate) fn draw(&mut self, ui: &mut egui::Ui) {
        let orion_state = self.orion_state.lock().unwrap().clone();

        Self::draw_base_graphics(ui);
        Self::draw_labels(ui);
        Self::draw_levels(ui, &orion_state.mixer_levels);
        Self::draw_pans_solos_mutes(ui, &orion_state.mixer_pans_solos_mutes);
    }

    fn draw_base_graphics(ui: &mut egui::Ui) {
        render_asset!(ui, 0, 222, "Mixer/default/background", 1.0, 0.0);

        for channel in 0..32 {
            let pos_x = 7 + channel * 35;

            if channel % 2 == 0 {
                render_asset!(ui, pos_x, 640, "LinkButton/default/released", 1.0, 0.0);
            }
        }

        render_asset!(
            ui,
            1136,
            356,
            "MasterMixerSlider/default/layers/handle",
            1.0,
            0.0
        );
        render_asset!(ui, 1136, 639, "MuteButton/default/released", 1.0, 0.0);
    }

    fn draw_levels(ui: &mut egui::Ui, levels: &[u8; 33]) {
        for channel in 0..32 {
            let level = levels[channel + 1];

            let base_pos_x = channel as f32 * ControlPanelMixerDimensions::STEP_X;

            let slider_pos_x = base_pos_x as u16 + ControlPanelMixerDimensions::SLIDER_START_X;
            let slider_pos_y = ControlPanelMixerDimensions::SLIDER_START_Y
                + level as u16 * ControlPanelMixerDimensions::SLIDER_DEPTH_Y / 95;

            render_asset!(
                ui,
                slider_pos_x,
                slider_pos_y,
                "MixerSlider/default/layers/handle",
                1.0,
                0.0
            );

            let text = if level == 0x5f {
                "-inf".to_string()
            } else {
                (-(level as i8)).to_string()
            };

            ui.painter().text(
                egui::pos2(
                    base_pos_x + ControlPanelMixerDimensions::TEXT_X,
                    ControlPanelMixerDimensions::LEVEL_TEXT_Y,
                ),
                egui::Align2::CENTER_CENTER,
                text,
                egui::FontId::proportional(ControlPanelMixerDimensions::FONT_SIZE_VALUE),
                ControlPanelMixerColors::VALUE_TEXT,
            );
        }
    }

    fn draw_pans_solos_mutes(ui: &mut egui::Ui, pans_solos_mutes: &[u8; 33]) {
        let any_solo = pans_solos_mutes.iter().any(|value| value & 0x80 != 0);

        for channel in 0..32 {
            let pan_solo_mute = pans_solos_mutes[channel + 1];

            let pan = (pan_solo_mute & 0x3f) as i8 - 32;
            let solo = pan_solo_mute & 0x80 != 0;
            let mute = pan_solo_mute & 0x40 != 0;

            let base_pos_x = channel as f32 * ControlPanelMixerDimensions::STEP_X;

            let text_x = base_pos_x + ControlPanelMixerDimensions::TEXT_X;

            ui.painter().text(
                egui::pos2(text_x, ControlPanelMixerDimensions::PAN_TEXT_Y),
                egui::Align2::CENTER_CENTER,
                pan.to_string(),
                egui::FontId::proportional(ControlPanelMixerDimensions::FONT_SIZE_VALUE),
                ControlPanelMixerColors::VALUE_TEXT,
            );

            let knob_pos_x = base_pos_x as u16 + ControlPanelMixerDimensions::PAN_KNOB_X;
            let knob_rad = (pan as f32 / 30.0) * ControlPanelMixerDimensions::PAN_RAD;

            render_asset!(
                ui,
                knob_pos_x,
                ControlPanelMixerDimensions::PAN_KNOB_Y,
                "MixerPan/default/layers/pointer",
                1.0,
                knob_rad
            );

            let solo_mute_x = base_pos_x as u16 + ControlPanelMixerDimensions::SOLO_MUTE_X;

            if solo {
                render_asset!(
                    ui,
                    solo_mute_x,
                    ControlPanelMixerDimensions::SOLO_Y,
                    "SoloButton/default/pressed",
                    1.0,
                    0.0
                );
            } else {
                render_asset!(
                    ui,
                    solo_mute_x,
                    ControlPanelMixerDimensions::SOLO_Y,
                    "SoloButton/default/released",
                    1.0,
                    0.0
                );
            }

            if !solo && any_solo || mute {
                render_asset!(
                    ui,
                    solo_mute_x,
                    ControlPanelMixerDimensions::MUTE_Y,
                    "MuteButton/default/pressed",
                    1.0,
                    0.0
                );
            } else {
                render_asset!(
                    ui,
                    solo_mute_x,
                    ControlPanelMixerDimensions::MUTE_Y,
                    "MuteButton/default/released",
                    1.0,
                    0.0
                );
            }
        }
    }

    fn draw_labels(ui: &mut egui::Ui) {
        for channel in 0..32 {
            let pos_x = ControlPanelMixerDimensions::LABEL_X
                + channel as f32 * ControlPanelMixerDimensions::STEP_X;

            let rect = egui::Rect::from_min_size(
                egui::pos2(pos_x, ControlPanelMixerDimensions::LABEL_Y),
                egui::vec2(
                    ControlPanelMixerDimensions::LABEL_WIDTH,
                    ControlPanelMixerDimensions::LABEL_HEIGHT,
                ),
            );

            ui.painter()
                .rect_filled(rect, 0.0, ControlPanelMixerColors::LABEL);

            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                (channel + 1).to_string(),
                egui::FontId::proportional(ControlPanelMixerDimensions::FONT_SIZE_LABEL),
                ControlPanelMixerColors::LABEL_TEXT,
            );
        }

        ui.painter().text(
            egui::pos2(
                ControlPanelMixerDimensions::MASTER_TEXT_X,
                ControlPanelMixerDimensions::MASTER_TEXT_Y,
            ),
            egui::Align2::CENTER_CENTER,
            "0",
            egui::FontId::proportional(ControlPanelMixerDimensions::FONT_SIZE_VALUE),
            ControlPanelMixerColors::VALUE_TEXT,
        );
    }
}
