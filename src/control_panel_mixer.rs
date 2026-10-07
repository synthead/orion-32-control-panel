use std::sync::Arc;
use std::sync::Mutex;

use crate::orion_hid::OrionHid;
use crate::orion_state::OrionState;

// use crate::control_panel_helpers::ControlPanelHelpers;

use crate::render_asset;

struct ControlPanelMixerColors;

impl ControlPanelMixerColors {
    const LABEL: egui::Color32 = egui::Color32::from_rgb(0x20, 0xa4, 0xf7);
    const FONT_LABEL: egui::Color32 = egui::Color32::WHITE;
    const FONT_VALUE: egui::Color32 = egui::Color32::from_rgb(0x95, 0x95, 0x95);
}

struct ControlPanelMixerDimensions;

impl ControlPanelMixerDimensions {
    const STEP_X: f32 = 35.0;

    const LABEL_X: f32 = 5.0;
    const LABEL_Y: f32 = 239.0;
    const LABEL_WIDTH: f32 = 33.0;
    const LABEL_HEIGHT: f32 = 13.0;

    const SLIDER_START_X: u16 = 11;
    const SLIDER_START_Y: u16 = 366;
    const SLIDER_DEPTH_Y: u16 = 178;

    const TEXT_X: f32 = 21.0;
    const LEVEL_Y: f32 = 348.0;

    const PAN_Y_OFFSET: f32 = 17.0;
    const LEVEL_Y_OFFSET: f32 = 96.0;

    const MASTER_FONT_X: f32 = 1150.0;
    const MASTER_FONT_Y: f32 = 342.0;

    const FONT_SIZE_LABEL: f32 = 10.0;
    const FONT_SIZE_VALUE: f32 = 12.0;

    const POINTER_SWEEP_RAD: f32 = 17.26;
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
        Self::draw_mixer_levels(ui, &orion_state.mixer_levels);
    }

    fn draw_base_graphics(ui: &mut egui::Ui) {
        render_asset!(ui, 0, 222, "Mixer/default/background", 1.0, 0.0);

        for mixer_channel in 0..32 {
            let pos_x = 7 + mixer_channel * 35;

            render_asset!(
                ui,
                pos_x - 2,
                283,
                "MixerPan/default/layers/pointer",
                1.0,
                0.0
            );

            render_asset!(ui, pos_x, 604, "SoloButton/default/released", 1.0, 0.0);
            render_asset!(ui, pos_x, 622, "MuteButton/default/released", 1.0, 0.0);

            if mixer_channel % 2 == 0 {
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

    fn draw_mixer_levels(ui: &mut egui::Ui, mixer_levels: &[u8; 33]) {
        for mixer_channel in 0..32 {
            let mixer_level = mixer_levels[mixer_channel + 1];

            let base_pos_x = mixer_channel as f32 * ControlPanelMixerDimensions::STEP_X;

            let slider_pos_x = base_pos_x as u16 + ControlPanelMixerDimensions::SLIDER_START_X;
            let slider_pos_y = ControlPanelMixerDimensions::SLIDER_START_Y
                + mixer_level as u16 * ControlPanelMixerDimensions::SLIDER_DEPTH_Y / 95;

            render_asset!(
                ui,
                slider_pos_x,
                slider_pos_y,
                "MixerSlider/default/layers/handle",
                1.0,
                0.0
            );

            let text = if mixer_level == 0x5f { "-inf".to_string() } else { (-(mixer_level as i8)).to_string() };

            ui.painter().text(
                egui::pos2(
                    base_pos_x + ControlPanelMixerDimensions::TEXT_X,
                    ControlPanelMixerDimensions::LEVEL_Y,
                ),
                egui::Align2::CENTER_CENTER,
                text,
                egui::FontId::proportional(ControlPanelMixerDimensions::FONT_SIZE_VALUE),
                ControlPanelMixerColors::FONT_VALUE,
            );
        }
    }

    fn draw_labels(ui: &mut egui::Ui) {
        for mixer_channel in 0..32 {
            let pos_x = ControlPanelMixerDimensions::LABEL_X
                + mixer_channel as f32 * ControlPanelMixerDimensions::STEP_X;

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
                (mixer_channel + 1).to_string(),
                egui::FontId::proportional(ControlPanelMixerDimensions::FONT_SIZE_LABEL),
                ControlPanelMixerColors::FONT_LABEL,
            );

            ui.painter().text(
                rect.center_bottom() + egui::vec2(0.0, ControlPanelMixerDimensions::PAN_Y_OFFSET),
                egui::Align2::CENTER_CENTER,
                "0",
                egui::FontId::proportional(ControlPanelMixerDimensions::FONT_SIZE_VALUE),
                ControlPanelMixerColors::FONT_VALUE,
            );
        }

        ui.painter().text(
            egui::pos2(
                ControlPanelMixerDimensions::MASTER_FONT_X,
                ControlPanelMixerDimensions::MASTER_FONT_Y,
            ),
            egui::Align2::CENTER_CENTER,
            "0",
            egui::FontId::proportional(ControlPanelMixerDimensions::FONT_SIZE_VALUE),
            ControlPanelMixerColors::FONT_VALUE,
        );
    }
}
