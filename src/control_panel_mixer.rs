use std::sync::Arc;
use std::sync::Mutex;

use crate::orion_hid::OrionHid;
use crate::orion_state::OrionState;

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
    const PAN_KNOB_RAD: f32 = 2.35;

    const SLIDER_X: u16 = 11;
    const SLIDER_Y: u16 = 366;
    const SLIDER_TRAVEL: u16 = 178;

    const SLIDER_HITBOX_X: f32 = 11.0;
    const SLIDER_HITBOX_Y: f32 = 366.0;
    const SLIDER_HITBOX_WIDTH: f32 = 22.0;
    const SLIDER_HITBOX_HEIGHT: f32 = 222.0;

    const SOLO_MUTE_X: u16 = 7;
    const SOLO_Y: u16 = 604;
    const MUTE_Y: u16 = 622;

    const TEXT_X: f32 = 21.0;
    const PAN_TEXT_Y: f32 = 270.0;
    const LEVEL_TEXT_Y: f32 = 348.0;

    const MASTER_TEXT_X: f32 = 1150.0;
    const MASTER_TEXT_Y: f32 = 342.0;

    const MSATER_SLIDER_X: u16 = 1136;
    const MSATER_SLIDER_Y: u16 = 356;
    const MSATER_SLIDER_TRAVEL: u16 = 233;

    const MASTER_MUTE_X: u16 = 1136;
    const MASTER_MUTE_Y: u16 = 639;
}

pub(crate) struct ControlPanelMixer {
    orion_state: Arc<Mutex<OrionState>>,
    orion_hid: Arc<Mutex<OrionHid>>,

    drag_offset: f32,
    drag_value: u8,
}

impl ControlPanelMixer {
    pub(crate) fn new(
        orion_state: Arc<Mutex<OrionState>>,
        orion_hid: Arc<Mutex<OrionHid>>,
    ) -> Self {
        Self {
            orion_state,
            orion_hid,

            drag_offset: 0.0,
            drag_value: 0,
        }
    }

    pub(crate) fn draw(&mut self, ui: &mut egui::Ui) {
        let orion_state = self.orion_state.lock().unwrap().clone();

        Self::draw_base_graphics(ui);
        Self::draw_labels(ui);

        self.draw_levels(
            ui,
            &orion_state.mixer_levels,
            &orion_state.mixer_pans_solos_mutes,
        );

        self.draw_pans_solos_mutes(
            ui,
            &orion_state.mixer_levels,
            &orion_state.mixer_pans_solos_mutes,
        );

        self.draw_master(
            ui,
            orion_state.mixer_levels[0],
            orion_state.mixer_pans_solos_mutes[0],
        );
    }

    fn draw_base_graphics(ui: &mut egui::Ui) {
        render_asset!(ui, 0, 222, "Mixer/default/background", 1.0, 0.0, None);

        for channel in 0..32 {
            let pos_x = 7 + channel * 35;

            if channel % 2 == 0 {
                render_asset!(
                    ui,
                    pos_x,
                    640,
                    "LinkButton/default/released",
                    1.0,
                    0.0,
                    None
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
    }

    fn draw_levels(&mut self, ui: &mut egui::Ui, levels: &[u8; 33], pans_solos_mutes: &[u8; 33]) {
        for channel in 1..33 {
            let mut level = levels[channel];

            let base_pos_x = (channel - 1) as f32 * ControlPanelMixerDimensions::STEP_X;
            let slider_pos_x = base_pos_x as u16 + ControlPanelMixerDimensions::SLIDER_X;

            let hitbox_rect = egui::Rect::from_min_size(
                egui::pos2(
                    base_pos_x + ControlPanelMixerDimensions::SLIDER_HITBOX_X,
                    ControlPanelMixerDimensions::SLIDER_HITBOX_Y,
                ),
                egui::vec2(
                    ControlPanelMixerDimensions::SLIDER_HITBOX_WIDTH,
                    ControlPanelMixerDimensions::SLIDER_HITBOX_HEIGHT,
                ),
            );

            let response = ui.allocate_rect(hitbox_rect, egui::Sense::click_and_drag());

            if response.drag_started() {
                self.drag_offset = 0.0;
            }

            if response.drag_stopped() {
                level = self.drag_value;
                let pan_solo_mute = pans_solos_mutes[channel];

                self.orion_hid.lock().unwrap().queue_mixer_channel(
                    channel as u8,
                    level,
                    pan_solo_mute,
                );
            }

            let mut slider_pos_y = ControlPanelMixerDimensions::SLIDER_Y
                + level as u16 * ControlPanelMixerDimensions::SLIDER_TRAVEL / 95;

            if response.dragged() {
                let slider_min_y = ControlPanelMixerDimensions::SLIDER_Y as f32;
                let slider_max_y = slider_min_y + ControlPanelMixerDimensions::SLIDER_TRAVEL as f32;

                let raw_y =
                    slider_pos_y as f32 + response.total_drag_delta().unwrap().y + self.drag_offset;

                if raw_y < slider_min_y {
                    self.drag_offset += slider_min_y - raw_y;
                    slider_pos_y = slider_min_y as u16;
                } else if raw_y > slider_max_y {
                    self.drag_offset += slider_max_y - raw_y;
                    slider_pos_y = slider_max_y as u16;
                } else {
                    slider_pos_y = raw_y as u16;
                }

                level = ((slider_pos_y - ControlPanelMixerDimensions::SLIDER_Y) * 95
                    / ControlPanelMixerDimensions::SLIDER_TRAVEL) as u8;

                slider_pos_y = ControlPanelMixerDimensions::SLIDER_Y
                    + level as u16 * ControlPanelMixerDimensions::SLIDER_TRAVEL / 95;

                self.drag_value = level;
            }

            render_asset!(
                ui,
                slider_pos_x,
                slider_pos_y,
                "MixerSlider/default/layers/handle",
                1.0,
                0.0,
                None
            );

            ui.painter().text(
                egui::pos2(
                    base_pos_x + ControlPanelMixerDimensions::TEXT_X,
                    ControlPanelMixerDimensions::LEVEL_TEXT_Y,
                ),
                egui::Align2::CENTER_CENTER,
                Self::text_for_level(level),
                egui::FontId::proportional(ControlPanelMixerDimensions::FONT_SIZE_VALUE),
                ControlPanelMixerColors::VALUE_TEXT,
            );
        }
    }

    fn draw_pans_solos_mutes(
        &self,
        ui: &mut egui::Ui,
        levels: &[u8; 33],
        pans_solos_mutes: &[u8; 33],
    ) {
        let any_solo = pans_solos_mutes.iter().any(|value| value & 0x80 != 0);

        for channel in 1..33 {
            let pan_solo_mute = pans_solos_mutes[channel];

            let pan = (pan_solo_mute & 0x3f) as i8 - 32;
            let solo = pan_solo_mute & 0x80 != 0;
            let mute = pan_solo_mute & 0x40 != 0;

            let base_pos_x = (channel - 1) as f32 * ControlPanelMixerDimensions::STEP_X;

            let text_x = base_pos_x + ControlPanelMixerDimensions::TEXT_X;

            ui.painter().text(
                egui::pos2(text_x, ControlPanelMixerDimensions::PAN_TEXT_Y),
                egui::Align2::CENTER_CENTER,
                pan.to_string(),
                egui::FontId::proportional(ControlPanelMixerDimensions::FONT_SIZE_VALUE),
                ControlPanelMixerColors::VALUE_TEXT,
            );

            let knob_pos_x = base_pos_x as u16 + ControlPanelMixerDimensions::PAN_KNOB_X;
            let knob_rad = (pan as f32 / 30.0) * ControlPanelMixerDimensions::PAN_KNOB_RAD;

            render_asset!(
                ui,
                knob_pos_x,
                ControlPanelMixerDimensions::PAN_KNOB_Y,
                "MixerPan/default/layers/pointer",
                1.0,
                knob_rad,
                None
            );

            let solo_mute_x = base_pos_x as u16 + ControlPanelMixerDimensions::SOLO_MUTE_X;

            let solo_button = if solo {
                render_asset!(
                    ui,
                    solo_mute_x,
                    ControlPanelMixerDimensions::SOLO_Y,
                    "SoloButton/default/pressed",
                    1.0,
                    0.0,
                    Some(egui::Sense::click())
                )
            } else {
                render_asset!(
                    ui,
                    solo_mute_x,
                    ControlPanelMixerDimensions::SOLO_Y,
                    "SoloButton/default/released",
                    1.0,
                    0.0,
                    Some(egui::Sense::click())
                )
            };

            let mute_button = if !solo && any_solo || mute {
                render_asset!(
                    ui,
                    solo_mute_x,
                    ControlPanelMixerDimensions::MUTE_Y,
                    "MuteButton/default/pressed",
                    1.0,
                    0.0,
                    Some(egui::Sense::click())
                )
            } else {
                render_asset!(
                    ui,
                    solo_mute_x,
                    ControlPanelMixerDimensions::MUTE_Y,
                    "MuteButton/default/released",
                    1.0,
                    0.0,
                    Some(egui::Sense::click())
                )
            };

            let level = levels[channel];

            if solo_button.clicked() {
                self.orion_hid.lock().unwrap().queue_mixer_channel(
                    channel as u8,
                    level,
                    pan_solo_mute ^ 0x80,
                );
            }

            if mute_button.clicked() {
                self.orion_hid.lock().unwrap().queue_mixer_channel(
                    channel as u8,
                    level,
                    pan_solo_mute ^ 0x40,
                );
            }
        }
    }

    fn draw_master(&self, ui: &mut egui::Ui, level: u8, mute: u8) {
        let slider_pos_y = ControlPanelMixerDimensions::MSATER_SLIDER_Y
            + level as u16 * ControlPanelMixerDimensions::MSATER_SLIDER_TRAVEL / 95;

        render_asset!(
            ui,
            ControlPanelMixerDimensions::MSATER_SLIDER_X,
            slider_pos_y,
            "MasterMixerSlider/default/layers/handle",
            1.0,
            0.0,
            None
        );

        ui.painter().text(
            egui::pos2(
                ControlPanelMixerDimensions::MASTER_TEXT_X,
                ControlPanelMixerDimensions::MASTER_TEXT_Y,
            ),
            egui::Align2::CENTER_CENTER,
            Self::text_for_level(level),
            egui::FontId::proportional(ControlPanelMixerDimensions::FONT_SIZE_VALUE),
            ControlPanelMixerColors::VALUE_TEXT,
        );

        let mute_button = if mute & 0x40 != 0 {
            render_asset!(
                ui,
                ControlPanelMixerDimensions::MASTER_MUTE_X,
                ControlPanelMixerDimensions::MASTER_MUTE_Y,
                "MuteButton/default/pressed",
                1.0,
                0.0,
                Some(egui::Sense::click())
            )
        } else {
            render_asset!(
                ui,
                ControlPanelMixerDimensions::MASTER_MUTE_X,
                ControlPanelMixerDimensions::MASTER_MUTE_Y,
                "MuteButton/default/released",
                1.0,
                0.0,
                Some(egui::Sense::click())
            )
        };

        if mute_button.clicked() {
            self.orion_hid
                .lock()
                .unwrap()
                .queue_mixer_channel(0, level, mute ^ 0x40);
        }
    }

    fn text_for_level(level: u8) -> String {
        if level == 0x5f {
            return "-inf".to_string();
        };

        (-(level as i8)).to_string()
    }
}
