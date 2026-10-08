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
    const LABEL_Y: f32 = 243.0;
    const LABEL_WIDTH: f32 = 33.0;
    const LABEL_HEIGHT: f32 = 13.0;

    const PAN_KNOB_X: u16 = 5;
    const PAN_KNOB_Y: u16 = 287;
    const PAN_KNOB_RAD: f32 = 2.35;

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

    const TEXT_X: f32 = 21.0;
    const PAN_TEXT_Y: f32 = 274.0;
    const LEVEL_TEXT_Y: f32 = 352.0;

    const MASTER_TEXT_X: f32 = 1150.0;
    const MASTER_TEXT_Y: f32 = 346.0;

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

    fn draw_levels(&mut self, ui: &mut egui::Ui, levels: &[u8; 33], pans_solos_mutes: &[u8; 33]) {
        for channel in 1..33 {
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

            let level = self.level_with_drag(
                ui,
                levels[channel],
                0x00,
                0x5f,
                ControlPanelMixerDimensions::SLIDER_TRAVEL,
                hitbox_rect,
                |orion_hid, level| {
                    orion_hid.queue_mixer_channel(channel as u8, level, pans_solos_mutes[channel]);
                },
            );

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

            let pan = pan_solo_mute & 0x3f;
            let solo = pan_solo_mute & 0x80 != 0;
            let mute = pan_solo_mute & 0x40 != 0;

            let base_pos_x = (channel - 1) as f32 * ControlPanelMixerDimensions::STEP_X;

            let text_x = base_pos_x + ControlPanelMixerDimensions::TEXT_X;
            let pan_float = (pan as f32 - 32.0);

            ui.painter().text(
                egui::pos2(text_x, ControlPanelMixerDimensions::PAN_TEXT_Y),
                egui::Align2::CENTER_CENTER,
                pan_float.to_string(),
                egui::FontId::proportional(ControlPanelMixerDimensions::FONT_SIZE_VALUE),
                ControlPanelMixerColors::VALUE_TEXT,
            );

            let knob_pos_x = base_pos_x as u16 + ControlPanelMixerDimensions::PAN_KNOB_X;
            let knob_rad = pan_float * ControlPanelMixerDimensions::PAN_KNOB_RAD / 30.0;

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

            let mute_button = if !solo && (any_solo || mute) {
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

        let level = self.level_with_drag(
            ui,
            level,
            0x00,
            0x5f,
            ControlPanelMixerDimensions::MASTER_SLIDER_TRAVEL,
            hitbox_rect,
            |orion_hid, level| {
                orion_hid.queue_mixer_channel(0, level, mute);
            },
        );

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

    fn level_with_drag(
        &mut self,
        ui: &mut egui::Ui,
        level: u8,
        level_min: u8,
        level_max: u8,
        drag_travel: u16,
        hitbox_rect: egui::Rect,
        mut drag_stopped_callback: impl FnMut(&mut OrionHid, u8),
    ) -> u8 {
        let response = ui.allocate_rect(hitbox_rect, egui::Sense::click_and_drag());

        if response.drag_started() {
            self.drag_offset = 0.0;
            self.drag_value = level;
        }

        if response.dragged() {
            let level_span = (level_max - level_min) as f32;
            let pixels_per_level = drag_travel as f32 / level_span;
            let level_offset = (level - level_min) as f32;

            let current_pos = level_offset * pixels_per_level;
            let raw_pos = current_pos + response.total_drag_delta().unwrap().y + self.drag_offset;
            let clamped_pos = raw_pos.clamp(0.0, drag_travel as f32);

            let level_unclamped = (level_min as f32 + clamped_pos / pixels_per_level).round() as u8;

            self.drag_offset += clamped_pos - raw_pos;
            self.drag_value = level_unclamped.clamp(level_min, level_max);

            return self.drag_value;
        }

        if response.drag_stopped() {
            let mut orion_hid = self.orion_hid.lock().unwrap();

            drag_stopped_callback(&mut orion_hid, self.drag_value);

            return self.drag_value;
        }

        level
    }

    fn text_for_level(level: u8) -> String {
        if level == 0x5f {
            return "-inf".to_string();
        };

        (-(level as i8)).to_string()
    }
}
