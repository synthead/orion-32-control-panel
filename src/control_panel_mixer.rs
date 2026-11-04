use std::sync::Arc;
use std::sync::Mutex;

use crate::constants::{WidgetColors, WidgetDimensions};

use crate::orion_hid::OrionHid;
use crate::orion_state::OrionState;

use crate::drag_value::DragValue;
use crate::text_editor::TextEditor;

use crate::control_panel_helpers::ControlPanelHelpers;

use crate::render_asset;

struct ControlPanelMixerDimensions;

impl ControlPanelMixerDimensions {
    const STEP_X: f32 = 35.0;

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

    const SOLO_MUTE_LINK_X: u16 = 7;
    const SOLO_Y: u16 = 608;
    const MUTE_Y: u16 = 626;
    const LINK_Y: u16 = 644;

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
    orion_hid: Arc<Mutex<OrionHid>>,

    text_editor: Arc<Mutex<TextEditor>>,
    drag_value: Arc<Mutex<DragValue>>,

    linked_channels: [bool; 16],
}

impl ControlPanelMixer {
    const SOLO_BIT: u8 = 0x80;
    const MUTE_BIT: u8 = 0x40;

    pub(crate) fn new(
        orion_hid: Arc<Mutex<OrionHid>>,
        text_editor: Arc<Mutex<TextEditor>>,
        drag_value: Arc<Mutex<DragValue>>,
    ) -> Self {
        Self {
            orion_hid,

            text_editor,
            drag_value,

            linked_channels: [false; 16],
        }
    }

    pub(crate) fn draw(&mut self, ui: &mut egui::Ui, orion_state: &mut OrionState) {
        orion_state.calculate_channel_counts();

        Self::draw_base_graphics(ui);
        Self::draw_labels(ui, orion_state);

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
    }

    fn draw_labels(ui: &mut egui::Ui, orion_state: &OrionState) {
        for channel in 0..32 {
            let route = orion_state.routes[6][channel];
            let label = orion_state.label_for_route(route, true);
            let color = orion_state.color_for_route(route, true);

            let pos_x = ControlPanelMixerDimensions::LABEL_X
                + channel as f32 * ControlPanelMixerDimensions::STEP_X;

            let rect = egui::Rect::from_min_size(
                egui::pos2(pos_x, ControlPanelMixerDimensions::LABEL_Y),
                egui::vec2(
                    ControlPanelMixerDimensions::LABEL_WIDTH,
                    ControlPanelMixerDimensions::LABEL_HEIGHT,
                ),
            );

            ui.painter().rect_filled(rect, 0.0, color);

            ui.painter().with_clip_rect(rect).text(
                rect.center() - egui::vec2(0.0, 1.0),
                egui::Align2::CENTER_CENTER,
                &label,
                egui::FontId::proportional(WidgetDimensions::TEXT_SIZE),
                WidgetColors::TEXT_COLOR_LABEL,
            );
        }
    }

    fn draw_channels(&mut self, ui: &mut egui::Ui, levels: &[u8; 33], pans_solos_mutes: &[u8; 33]) {
        let any_solo = pans_solos_mutes
            .iter()
            .any(|value| value & Self::SOLO_BIT != 0);

        let mut edit_permitted = true;
        let mut linked_levels: [Option<(u8, bool)>; 16] = [None; 16];

        for channel in 1..33 {
            let channel_index = channel - 1;
            let left_channel = channel_index | 1;
            let right_channel = left_channel + 1;

            let linked_index = channel_index / 2;
            let linked_state = self.linked_channels[linked_index];

            let linked_channel = if channel == left_channel {
                right_channel
            } else {
                left_channel
            };

            let pair_has_solo = pans_solos_mutes[left_channel] & Self::SOLO_BIT != 0
                || pans_solos_mutes[right_channel] & Self::SOLO_BIT != 0;

            let base_pos_x = channel_index as f32 * ControlPanelMixerDimensions::STEP_X;
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

            let (level, level_should_write) = if linked_state {
                if channel == left_channel {
                    let linked_hitbox_rect = egui::Rect::from_min_size(
                        egui::pos2(
                            base_pos_x + ControlPanelMixerDimensions::SLIDER_HITBOX_X,
                            ControlPanelMixerDimensions::SLIDER_HITBOX_Y,
                        ),
                        egui::vec2(
                            ControlPanelMixerDimensions::STEP_X
                                + ControlPanelMixerDimensions::SLIDER_HITBOX_WIDTH,
                            ControlPanelMixerDimensions::SLIDER_HITBOX_HEIGHT,
                        ),
                    );

                    let result = self.drag_value.lock().unwrap().value_with_drag(
                        ui,
                        linked_hitbox_rect,
                        levels[left_channel],
                        0x00,
                        0x5f,
                        ControlPanelMixerDimensions::SLIDER_TRAVEL,
                        -1.0,
                        false,
                    );

                    linked_levels[linked_index] = Some(result);

                    result
                } else {
                    linked_levels[linked_index].unwrap_or((levels[left_channel], false))
                }
            } else {
                self.drag_value.lock().unwrap().value_with_drag(
                    ui,
                    slider_hitbox_rect,
                    levels[channel],
                    0x00,
                    0x5f,
                    ControlPanelMixerDimensions::SLIDER_TRAVEL,
                    -1.0,
                    false,
                )
            };

            if level_should_write && (!linked_state || channel == left_channel) {
                let mut orion_hid = self.orion_hid.lock().unwrap();

                orion_hid.queue_mixer_channel(channel as u8, level, pans_solos_mutes[channel]);

                if linked_state {
                    orion_hid.queue_mixer_channel(
                        linked_channel as u8,
                        level,
                        pans_solos_mutes[linked_channel],
                    );
                }
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
                egui::Id::new(("level", channel as u32)),
                &level_text,
                WidgetDimensions::TEXT_SIZE_MIXER_VALUE,
                WidgetColors::TEXT_COLOR,
            );

            if edit_permitted {
                if let Some((level_text, tab)) = level_edit_result {
                    if let Some(parsed_level) = Self::level_for_text(&level_text) {
                        let mut orion_hid = self.orion_hid.lock().unwrap();

                        orion_hid.queue_mixer_channel(
                            channel as u8,
                            parsed_level,
                            pans_solos_mutes[channel],
                        );

                        if linked_state {
                            orion_hid.queue_mixer_channel(
                                linked_channel as u8,
                                parsed_level,
                                pans_solos_mutes[linked_channel],
                            );
                        }
                    }

                    if let Some(tab) = tab {
                        let next_channel = (channel as isize - 1 + tab).rem_euclid(32) + 1;

                        self.text_editor
                            .lock()
                            .unwrap()
                            .set_editing_id(egui::Id::new(("level", next_channel as u32)));
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

            let solo = pan_solo_mute & Self::SOLO_BIT;
            let mute = pan_solo_mute & Self::MUTE_BIT;

            let (pan, pan_should_write) = self.drag_value.lock().unwrap().value_with_drag(
                ui,
                pan_hitbox_rect,
                pan_solo_mute & 0x3f,
                0x02,
                0x3e,
                ControlPanelMixerDimensions::PAN_KNOB_TRAVEL,
                1.0,
                false,
            );

            if pan_should_write {
                self.orion_hid.lock().unwrap().queue_mixer_channel(
                    channel as u8,
                    levels[channel],
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
                egui::Id::new(("pan", channel as u32)),
                &pan_float.to_string(),
                WidgetDimensions::TEXT_SIZE_MIXER_VALUE,
                WidgetColors::TEXT_COLOR,
            );

            if edit_permitted {
                if let Some((text, tab)) = pan_edit_result {
                    if let Some(parsed_pan) = Self::pan_for_text(&text) {
                        self.orion_hid.lock().unwrap().queue_mixer_channel(
                            channel as u8,
                            levels[channel],
                            parsed_pan | solo | mute,
                        );
                    }

                    if let Some(tab) = tab {
                        let next_channel = (channel as isize - 1 + tab).rem_euclid(32) + 1;

                        self.text_editor
                            .lock()
                            .unwrap()
                            .set_editing_id(egui::Id::new(("pan", next_channel as u32)));
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

            let solo_mute_link_x =
                base_pos_x as u16 + ControlPanelMixerDimensions::SOLO_MUTE_LINK_X;
            let solo_bool = solo != 0;
            let mute_bool = mute != 0;

            let solo_response = if solo_bool {
                render_asset!(
                    ui,
                    solo_mute_link_x,
                    ControlPanelMixerDimensions::SOLO_Y,
                    "SoloButton/default/pressed",
                    1.0,
                    0.0,
                    Some(egui::Sense::click())
                )
            } else {
                render_asset!(
                    ui,
                    solo_mute_link_x,
                    ControlPanelMixerDimensions::SOLO_Y,
                    "SoloButton/default/released",
                    1.0,
                    0.0,
                    Some(egui::Sense::click())
                )
            };

            let mute_response = if !solo_bool && (any_solo || mute_bool) {
                render_asset!(
                    ui,
                    solo_mute_link_x,
                    ControlPanelMixerDimensions::MUTE_Y,
                    "MuteButton/default/pressed",
                    1.0,
                    0.0,
                    Some(egui::Sense::click())
                )
            } else {
                render_asset!(
                    ui,
                    solo_mute_link_x,
                    ControlPanelMixerDimensions::MUTE_Y,
                    "MuteButton/default/released",
                    1.0,
                    0.0,
                    Some(egui::Sense::click())
                )
            };

            if ControlPanelHelpers::mouse_down_on(&solo_response) {
                let new_solo = if solo_bool { 0 } else { Self::SOLO_BIT };
                let channel_solo_mute = (pan_solo_mute & !Self::SOLO_BIT) | new_solo;

                let mut orion_hid = self.orion_hid.lock().unwrap();

                orion_hid.queue_mixer_channel(channel as u8, levels[channel], channel_solo_mute);

                if linked_state {
                    let linked_pan_solo_mute = pans_solos_mutes[linked_channel];
                    let linked_solo_mute = (linked_pan_solo_mute & !Self::SOLO_BIT) | new_solo;

                    orion_hid.queue_mixer_channel(
                        linked_channel as u8,
                        levels[linked_channel],
                        linked_solo_mute,
                    );
                }
            }

            if ControlPanelHelpers::mouse_down_on(&mute_response) && !any_solo {
                let new_mute = if mute_bool { 0 } else { Self::MUTE_BIT };
                let channel_solo_mute = (pan_solo_mute & !Self::MUTE_BIT) | new_mute;

                let mut orion_hid = self.orion_hid.lock().unwrap();

                orion_hid.queue_mixer_channel(channel as u8, levels[channel], channel_solo_mute);

                if linked_state {
                    let linked_pan_solo_mute = pans_solos_mutes[linked_channel];
                    let linked_solo_mute = (linked_pan_solo_mute & !Self::MUTE_BIT) | new_mute;

                    orion_hid.queue_mixer_channel(
                        linked_channel as u8,
                        levels[linked_channel],
                        linked_solo_mute,
                    );
                }
            }

            if channel % 2 == 1 {
                let linked_response = if linked_state {
                    render_asset!(
                        ui,
                        solo_mute_link_x,
                        ControlPanelMixerDimensions::LINK_Y,
                        "LinkButton/default/pressed",
                        1.0,
                        0.0,
                        Some(egui::Sense::click())
                    )
                } else {
                    render_asset!(
                        ui,
                        solo_mute_link_x,
                        ControlPanelMixerDimensions::LINK_Y,
                        "LinkButton/default/released",
                        1.0,
                        0.0,
                        Some(egui::Sense::click())
                    )
                };

                if ControlPanelHelpers::mouse_down_on(&linked_response) && !pair_has_solo {
                    if !linked_state {
                        let right_pan_solo_mute = (pans_solos_mutes[right_channel]
                            & !Self::MUTE_BIT)
                            | (pans_solos_mutes[left_channel] & Self::MUTE_BIT);

                        self.orion_hid.lock().unwrap().queue_mixer_channel(
                            right_channel as u8,
                            levels[left_channel],
                            right_pan_solo_mute,
                        );
                    }

                    self.linked_channels[linked_index] = !linked_state;
                }
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
            false,
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
            egui::Id::new(("level", 0u32)),
            &Self::text_for_level(level),
            WidgetDimensions::TEXT_SIZE_MIXER_VALUE,
            WidgetColors::TEXT_COLOR,
        );

        if let Some((text, None)) = edit_result {
            if let Some(parsed_level) = Self::level_for_text(&text) {
                self.orion_hid
                    .lock()
                    .unwrap()
                    .queue_mixer_channel(0, parsed_level, mute);
            }
        }

        let mute_response = if mute & Self::MUTE_BIT != 0 {
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

        if ControlPanelHelpers::mouse_down_on(&mute_response) {
            self.orion_hid
                .lock()
                .unwrap()
                .queue_mixer_channel(0, level, mute ^ Self::MUTE_BIT);
        }
    }

    fn text_for_level(level: u8) -> String {
        if level == 0x5f {
            return "-inf".to_string();
        }

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
