use std::sync::Arc;
use std::sync::Mutex;

use crate::orion_hid::OrionHid;
use crate::orion_state::OrionState;

use crate::asset_button::AssetButton;
use crate::drag_value::DragValue;
use crate::text_editor::TextEditor;

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
    const LABEL_Y: f32 = 243.0;
    const LABEL_WIDTH: f32 = 33.0;
    const LABEL_HEIGHT: f32 = 13.0;

    const PAN_KNOB_X: u16 = 5;
    const PAN_KNOB_Y: u16 = 287;
    const PAN_KNOB_RAD: f32 = 2.35;
    const PAN_KNOB_TRAVEL: u16 = 233;

    const PAN_KNOB_HITBOX_X: f32 = 5.0;
    const PAN_KNOB_HITBOX_Y: f32 = 284.0;
    const PAN_KNOB_HITBOX_WIDTH: f32 = 28.0;
    const PAN_KNOB_HITBOX_HEIGHT: f32 = 34.0;

    const SLIDER_X: u16 = 11;
    const SLIDER_Y: u16 = 370;
    const SLIDER_TRAVEL: u16 = 178;

    const SLIDER_HITBOX_X: f32 = 11.0;
    const SLIDER_HITBOX_Y: f32 = 370.0;
    const SLIDER_HITBOX_WIDTH: f32 = 22.0;
    const SLIDER_HITBOX_HEIGHT: f32 = 222.0;

    const SOLO_MUTE_X: u16 = 7;
    const SOLO_Y: u16 = 608;
    const MUTE_Y: u16 = 626;

    const TEXT_HITBOX_X: f32 = 9.0;
    const TEXT_HITBOX_Y_PAN: f32 = 266.0;
    const TEXT_HITBOX_Y_LEVEL: f32 = 344.0;
    const TEXT_HITBOX_WIDTH: f32 = 25.0;
    const TEXT_HITBOX_HEIGHT: f32 = 15.0;

    const MASTER_TEXT_HITBOX_X: f32 = 1138.0;
    const MASTER_TEXT_HITBOX_Y: f32 = 338.0;

    const MASTER_SLIDER_X: u16 = 1136;
    const MASTER_SLIDER_Y: u16 = 360;
    const MASTER_SLIDER_TRAVEL: u16 = 233;

    const MASTER_SLIDER_HITBOX_X: f32 = 1136.0;
    const MASTER_SLIDER_HITBOX_Y: f32 = 360.0;
    const MASTER_SLIDER_HITBOX_WIDTH: f32 = 32.0;
    const MASTER_SLIDER_HITBOX_HEIGHT: f32 = 283.0;

    const MASTER_MUTE_X: u16 = 1136;
    const MASTER_MUTE_Y: u16 = 643;
}

pub(crate) struct ControlPanelMixer {
    orion_state: Arc<Mutex<OrionState>>,
    orion_hid: Arc<Mutex<OrionHid>>,

    text_editor: Arc<Mutex<TextEditor>>,
    drag_value: Arc<Mutex<DragValue>>,
    asset_button: Arc<Mutex<AssetButton>>,
}

impl ControlPanelMixer {
    pub(crate) fn new(
        orion_state: Arc<Mutex<OrionState>>,
        orion_hid: Arc<Mutex<OrionHid>>,
        text_editor: Arc<Mutex<TextEditor>>,
        drag_value: Arc<Mutex<DragValue>>,
        asset_button: Arc<Mutex<AssetButton>>,
    ) -> Self {
        Self {
            orion_state,
            orion_hid,

            text_editor,
            drag_value,
            asset_button,
        }
    }

    pub(crate) fn draw(&mut self, ui: &mut egui::Ui) {
        let orion_state = self.orion_state.lock().unwrap().clone();

        Self::draw_base_graphics(ui);
        Self::draw_labels(ui);

        self.draw_channels(
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
        render_asset!(ui, 0, 226, "Mixer/default/background", 1.0, 0.0, None);

        for channel in 0..32 {
            let pos_x = 7 + channel * 35;

            if channel % 2 == 0 {
                render_asset!(
                    ui,
                    pos_x,
                    644,
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

    fn draw_channels(&mut self, ui: &mut egui::Ui, levels: &[u8; 33], pans_solos_mutes: &[u8; 33]) {
        let any_solo = pans_solos_mutes.iter().any(|value| value & 0x80 != 0);
        let mut edit_permitted = true;

        for channel in 1..33 {
            let base_pos_x = (channel - 1) as f32 * ControlPanelMixerDimensions::STEP_X;
            let slider_pos_x = base_pos_x as u16 + ControlPanelMixerDimensions::SLIDER_X;

            let slider_hitbox_rect = egui::Rect::from_min_size(
                egui::pos2(
                    base_pos_x + ControlPanelMixerDimensions::SLIDER_HITBOX_X,
                    ControlPanelMixerDimensions::SLIDER_HITBOX_Y,
                ),
                egui::vec2(
                    ControlPanelMixerDimensions::SLIDER_HITBOX_WIDTH,
                    ControlPanelMixerDimensions::SLIDER_HITBOX_HEIGHT,
                ),
            );

            let pan_solo_mute = pans_solos_mutes[channel];

            let (level, level_should_write) = self.drag_value.lock().unwrap().value_with_drag(
                ui,
                slider_hitbox_rect,
                levels[channel],
                0x00,
                0x5f,
                ControlPanelMixerDimensions::SLIDER_TRAVEL,
                -1.0,
            );

            if level_should_write {
                self.orion_hid.lock().unwrap().queue_mixer_channel(
                    channel as u8,
                    level,
                    pan_solo_mute,
                );
            }

            let slider_pos_y = ControlPanelMixerDimensions::SLIDER_Y
                + level as u16 * ControlPanelMixerDimensions::SLIDER_TRAVEL / 95;

            render_asset!(
                ui,
                slider_pos_x,
                slider_pos_y,
                "MixerSlider/default/layers/handle",
                1.0,
                0.0,
                None
            );

            let level_text = Self::text_for_level(level);

            let level_text_rect = egui::Rect::from_min_size(
                egui::pos2(
                    base_pos_x + ControlPanelMixerDimensions::TEXT_HITBOX_X,
                    ControlPanelMixerDimensions::TEXT_HITBOX_Y_LEVEL,
                ),
                egui::vec2(
                    ControlPanelMixerDimensions::TEXT_HITBOX_WIDTH,
                    ControlPanelMixerDimensions::TEXT_HITBOX_HEIGHT,
                ),
            );

            let level_edit_result = self.text_editor.lock().unwrap().draw(
                ui,
                level_text_rect,
                &format!("level_{channel}"),
                &level_text,
                ControlPanelMixerDimensions::FONT_SIZE_VALUE,
                ControlPanelMixerColors::VALUE_TEXT,
            );

            if edit_permitted {
                if let Some((level_text, tab)) = level_edit_result {
                    if let Some(parsed_level) = Self::level_for_text(&level_text) {
                        self.orion_hid.lock().unwrap().queue_mixer_channel(
                            channel as u8,
                            parsed_level,
                            pan_solo_mute,
                        );
                    }

                    if let Some(tab) = tab {
                        let next_channel = (channel as isize - 1 + tab).rem_euclid(32) + 1;

                        self.text_editor
                            .lock()
                            .unwrap()
                            .set_editing_id(&format!("level_{next_channel}"));
                    }

                    edit_permitted = false;
                }
            }

            let pan_hitbox_rect = egui::Rect::from_min_size(
                egui::pos2(
                    base_pos_x + ControlPanelMixerDimensions::PAN_KNOB_HITBOX_X,
                    ControlPanelMixerDimensions::PAN_KNOB_HITBOX_Y,
                ),
                egui::vec2(
                    ControlPanelMixerDimensions::PAN_KNOB_HITBOX_WIDTH,
                    ControlPanelMixerDimensions::PAN_KNOB_HITBOX_HEIGHT,
                ),
            );

            let solo = pan_solo_mute & 0x80;
            let mute = pan_solo_mute & 0x40;

            let (pan, pan_should_write) = self.drag_value.lock().unwrap().value_with_drag(
                ui,
                pan_hitbox_rect,
                pan_solo_mute & 0x3f,
                0x02,
                0x3e,
                ControlPanelMixerDimensions::PAN_KNOB_TRAVEL,
                1.0,
            );

            if pan_should_write {
                self.orion_hid.lock().unwrap().queue_mixer_channel(
                    channel as u8,
                    level,
                    pan | solo | mute,
                );
            }

            let pan_text_x = base_pos_x + ControlPanelMixerDimensions::TEXT_HITBOX_X;
            let pan_float = pan as f32 - 32.0;

            let pan_text_rect = egui::Rect::from_min_size(
                egui::pos2(pan_text_x, ControlPanelMixerDimensions::TEXT_HITBOX_Y_PAN),
                egui::vec2(
                    ControlPanelMixerDimensions::TEXT_HITBOX_WIDTH,
                    ControlPanelMixerDimensions::TEXT_HITBOX_HEIGHT,
                ),
            );

            let pan_edit_result = self.text_editor.lock().unwrap().draw(
                ui,
                pan_text_rect,
                &format!("pan_{channel}"),
                &pan_float.to_string(),
                ControlPanelMixerDimensions::FONT_SIZE_VALUE,
                ControlPanelMixerColors::VALUE_TEXT,
            );

            if edit_permitted {
                if let Some((text, tab)) = pan_edit_result {
                    if let Some(parsed_pan) = Self::pan_for_text(&text) {
                        self.orion_hid.lock().unwrap().queue_mixer_channel(
                            channel as u8,
                            level,
                            parsed_pan | solo | mute,
                        );
                    }

                    if let Some(tab) = tab {
                        let next_channel = (channel as isize - 1 + tab).rem_euclid(32) + 1;

                        self.text_editor
                            .lock()
                            .unwrap()
                            .set_editing_id(&format!("pan_{next_channel}"));
                    }

                    edit_permitted = false;
                }
            }

            let pan_knob_pos_x = base_pos_x as u16 + ControlPanelMixerDimensions::PAN_KNOB_X;
            let pan_knob_rad = pan_float * ControlPanelMixerDimensions::PAN_KNOB_RAD / 30.0;

            render_asset!(
                ui,
                pan_knob_pos_x,
                ControlPanelMixerDimensions::PAN_KNOB_Y,
                "MixerPan/default/layers/pointer",
                1.0,
                pan_knob_rad,
                None
            );

            let solo_mute_x = base_pos_x as u16 + ControlPanelMixerDimensions::SOLO_MUTE_X;
            let solo_bool = solo != 0;
            let mute_bool = mute != 0;

            let solo_response = render_asset!(
                ui,
                solo_mute_x,
                ControlPanelMixerDimensions::SOLO_Y,
                "SoloButton/default/released",
                1.0,
                0.0,
                Some(egui::Sense::click())
            );

            let (_solo_hover, solo_pressed, solo_clicked) = self
                .asset_button
                .lock()
                .unwrap()
                .pressed_when_down(ui, solo_response, solo_bool);

            if solo_pressed {
                render_asset!(
                    ui,
                    solo_mute_x,
                    ControlPanelMixerDimensions::SOLO_Y,
                    "SoloButton/default/pressed",
                    1.0,
                    0.0,
                    None
                );
            }

            if solo_clicked {
                self.orion_hid.lock().unwrap().queue_mixer_channel(
                    channel as u8,
                    level,
                    pan_solo_mute ^ 0x80,
                );
            }

            let mute_response = render_asset!(
                ui,
                solo_mute_x,
                ControlPanelMixerDimensions::MUTE_Y,
                "MuteButton/default/released",
                1.0,
                0.0,
                Some(egui::Sense::click())
            );

            let (_mute_hover, mute_pressed, mute_clicked) = self
                .asset_button
                .lock()
                .unwrap()
                .pressed_when_down(ui, mute_response, mute_bool);

            if !solo_bool && (any_solo || mute_pressed) {
                render_asset!(
                    ui,
                    solo_mute_x,
                    ControlPanelMixerDimensions::MUTE_Y,
                    "MuteButton/default/pressed",
                    1.0,
                    0.0,
                    None
                );
            }

            if mute_clicked && !any_solo {
                self.orion_hid.lock().unwrap().queue_mixer_channel(
                    channel as u8,
                    level,
                    pan_solo_mute ^ 0x40,
                );
            }
        }
    }

    fn draw_master(&mut self, ui: &mut egui::Ui, level: u8, mute: u8) {
        let hitbox_rect = egui::Rect::from_min_size(
            egui::pos2(
                ControlPanelMixerDimensions::MASTER_SLIDER_HITBOX_X,
                ControlPanelMixerDimensions::MASTER_SLIDER_HITBOX_Y,
            ),
            egui::vec2(
                ControlPanelMixerDimensions::MASTER_SLIDER_HITBOX_WIDTH,
                ControlPanelMixerDimensions::MASTER_SLIDER_HITBOX_HEIGHT,
            ),
        );

        let (level, should_write) = self.drag_value.lock().unwrap().value_with_drag(
            ui,
            hitbox_rect,
            level,
            0x00,
            0x5f,
            ControlPanelMixerDimensions::MASTER_SLIDER_TRAVEL,
            -1.0,
        );

        if should_write {
            self.orion_hid
                .lock()
                .unwrap()
                .queue_mixer_channel(0, level, mute);
        }

        let slider_pos_y = ControlPanelMixerDimensions::MASTER_SLIDER_Y
            + level as u16 * ControlPanelMixerDimensions::MASTER_SLIDER_TRAVEL / 95;

        render_asset!(
            ui,
            ControlPanelMixerDimensions::MASTER_SLIDER_X,
            slider_pos_y,
            "MasterMixerSlider/default/layers/handle",
            1.0,
            0.0,
            None
        );

        let text_rect = egui::Rect::from_min_size(
            egui::pos2(
                ControlPanelMixerDimensions::MASTER_TEXT_HITBOX_X,
                ControlPanelMixerDimensions::MASTER_TEXT_HITBOX_Y,
            ),
            egui::vec2(
                ControlPanelMixerDimensions::TEXT_HITBOX_WIDTH,
                ControlPanelMixerDimensions::TEXT_HITBOX_HEIGHT,
            ),
        );

        let edit_result = self.text_editor.lock().unwrap().draw(
            ui,
            text_rect,
            &format!("level_0"),
            &Self::text_for_level(level),
            ControlPanelMixerDimensions::FONT_SIZE_VALUE,
            ControlPanelMixerColors::VALUE_TEXT,
        );

        if let Some((text, None)) = edit_result {
            if let Some(parsed_level) = Self::level_for_text(&text) {
                self.orion_hid
                    .lock()
                    .unwrap()
                    .queue_mixer_channel(0, parsed_level, mute);
            }
        }

        let mute_state = mute & 0x40 != 0;

        let mute_response = render_asset!(
            ui,
            ControlPanelMixerDimensions::MASTER_MUTE_X,
            ControlPanelMixerDimensions::MASTER_MUTE_Y,
            "MuteButton/default/released",
            1.0,
            0.0,
            Some(egui::Sense::click())
        );

        let (_mute_hover, mute_pressed, mute_clicked) = self
            .asset_button
            .lock()
            .unwrap()
            .pressed_when_down(ui, mute_response, mute_state);

        if mute_pressed {
            render_asset!(
                ui,
                ControlPanelMixerDimensions::MASTER_MUTE_X,
                ControlPanelMixerDimensions::MASTER_MUTE_Y,
                "MuteButton/default/pressed",
                1.0,
                0.0,
                None
            );
        }

        if mute_clicked {
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

    fn level_for_text(text: &String) -> Option<u8> {
        let text_trimmed = text.trim();

        if text_trimmed == "-inf" {
            return Some(0x5f);
        }

        let text_parsed = match text_trimmed.parse::<i8>() {
            Ok(value) => value,
            Err(_) => return None,
        };

        if text_parsed < -95 || text_parsed > 0 {
            return None;
        }

        Some(text_parsed.abs() as u8)
    }

    fn pan_for_text(text: &String) -> Option<u8> {
        let text_trimmed = text.trim();

        let text_parsed = match text_trimmed.parse::<i8>() {
            Ok(value) => value,
            Err(_) => return None,
        };

        if text_parsed < -30 || text_parsed > 30 {
            return None;
        }

        Some((text_parsed + 0x20) as u8)
    }
}
