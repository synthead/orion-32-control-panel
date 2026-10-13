use std::sync::Arc;
use std::sync::Mutex;

use crate::orion_hid::OrionHid;
use crate::orion_state::OrionState;

use crate::drag_value::DragValue;
use crate::text_editor::TextEditor;

use crate::control_panel_helpers::ControlPanelHelpers;

use crate::render_asset;

struct ControlPanelSettingsDimensions;

impl ControlPanelSettingsDimensions {
    const VIEWPORT_X: f32 = 621.0;
    const VIEWPORT_Y: f32 = 392.0;

    const SETTINGS_LABEL_X: u16 = 530;
    const SETTINGS_LABEL_Y: u16 = 8;

    const WINDOW_BUTTON_CLOSE_X: u16 = 599;
    const WINDOW_BUTTON_CLOSE_Y: u16 = 6;

    const OSCILLATOR_MUTE_1_X: u16 = 61;
    const OSCILLATOR_MUTE_2_X: u16 = 550;
    const OSCILLATOR_MUTE_Y: u16 = 173;

    const TRIM_LINE_OUT_X: u16 = 211;
    const TRIM_ADC_X: u16 = 348;
    const TRIM_Y: u16 = 85;
    const TRIM_WIDTH: u8 = 68;

    const OSCILLATOR_FREQUENCY_1_X: u16 = 145;
    const OSCILLATOR_LEVEL_X: u16 = 282;
    const OSCILLATOR_FREQUENCY_2_X: u16 = 421;
    const OSCILLATOR_Y: u16 = 171;
    const OSCILLATOR_WIDTH: u8 = 68;

    const BRIGHTNESS_SLIDER_X: u16 = 20;
    const BRIGHTNESS_SLIDER_Y: u16 = 254;
    const BRIGHTNESS_SLIDER_TRAVEL: u16 = 69;

    const BRIGHTNESS_SLIDER_HITBOX_X: f32 = 20.0;
    const BRIGHTNESS_SLIDER_HITBOX_Y: f32 = 254.0;
    const BRIGHTNESS_SLIDER_HITBOX_WIDTH: f32 = 112.0;
    const BRIGHTNESS_SLIDER_HITBOX_HEIGHT: f32 = 21.0;

    const BRIGHTNESS_TEXT_HITBOX_X: f32 = 143.0;
    const BRIGHTNESS_TEXT_HITBOX_Y: f32 = 255.0;
    const BRIGHTNESS_TEXT_HITBOX_WIDTH: f32 = 52.0;
    const BRIGHTNESS_TEXT_HITBOX_HEIGHT: f32 = 18.0;

    const MADI_SMUX_IN_X: u16 = 398;
    const MADI_SMUX_OUT_X: u16 = 495;
    const MADI_SMUX_Y: u16 = 257;
}

pub(crate) struct ControlPanelSettings {
    orion_state: Arc<Mutex<OrionState>>,
    orion_hid: Arc<Mutex<OrionHid>>,

    text_editor: Arc<Mutex<TextEditor>>,
    drag_value: Arc<Mutex<DragValue>>,

    enabled: bool,
}

impl ControlPanelSettings {
    const WINDOW_TITLE: &str = "Orion 32 Settings";

    const FONT_COLOR: egui::Color32 = egui::Color32::from_rgb(0x95, 0x95, 0x95);
    const FONT_SIZE: f32 = 10.5;

    const OSC_FREQUENCY_SELECTIONS: [&str; 2] = ["1 kHz", "440 Hz"];

    const OSC_LEVEL_SELECTIONS: [&str; 4] = ["0 dBFS", "-6 dbFS", "-12 dbFS", "-18 dbFS"];

    const LINE_OUT_ADC_TRIM_SELECTIONS: [&str; 7] = [
        "20 dBu", "19 dBu", "18 dBu", "17 dBu", "16 dBu", "15 dBu", "14 dBu",
    ];

    pub(crate) fn new(
        orion_state: Arc<Mutex<OrionState>>,
        orion_hid: Arc<Mutex<OrionHid>>,
        text_editor: Arc<Mutex<TextEditor>>,
        drag_value: Arc<Mutex<DragValue>>,
    ) -> Self {
        Self {
            orion_state,
            orion_hid,

            text_editor,
            drag_value,

            enabled: false,
        }
    }

    pub(crate) fn enable(&mut self) {
        self.enabled = true;
    }

    fn ui_content(&mut self, ui: &mut egui::Ui) {
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ui, |ui| {
                let orion_state = self.orion_state.lock().unwrap().clone();

                self.draw_base_graphics(ui);
                self.draw_window_buttons(ui);

                self.draw_trims(ui, orion_state.trims);
                self.draw_oscillators(ui, orion_state.oscillators);
                self.draw_brightness(ui, orion_state.brightness);
                self.draw_madi_smux(ui, orion_state.madi_smux);
            });
    }

    pub(crate) fn draw(&mut self, ui: &mut egui::Ui) {
        if !self.enabled {
            return;
        }

        ui.ctx().show_viewport_immediate(
            egui::ViewportId::from_hash_of("control_panel_settings"),
            egui::ViewportBuilder::default()
                .with_title(Self::WINDOW_TITLE)
                .with_inner_size([
                    ControlPanelSettingsDimensions::VIEWPORT_X,
                    ControlPanelSettingsDimensions::VIEWPORT_Y,
                ])
                .with_decorations(false),
            |ctx, _class| self.ui_content(ctx),
        );
    }

    fn draw_base_graphics(&mut self, ui: &mut egui::Ui) {
        render_asset!(ui, 0, 0, "SettingsPanel/default/background", 1.0, 0.0, None);

        render_asset!(
            ui,
            ControlPanelSettingsDimensions::SETTINGS_LABEL_X,
            ControlPanelSettingsDimensions::SETTINGS_LABEL_Y,
            "SettingsLabel/default/background",
            1.0,
            0.0,
            None
        );
    }

    fn draw_window_buttons(&mut self, ui: &mut egui::Ui) {
        let close_response = render_asset!(
            ui,
            ControlPanelSettingsDimensions::WINDOW_BUTTON_CLOSE_X,
            ControlPanelSettingsDimensions::WINDOW_BUTTON_CLOSE_Y,
            "CloseButton/default/released",
            1.0,
            0.0,
            Some(egui::Sense::click())
        );

        if close_response.is_pointer_button_down_on() != close_response.hovered() {
            render_asset!(
                ui,
                ControlPanelSettingsDimensions::WINDOW_BUTTON_CLOSE_X,
                ControlPanelSettingsDimensions::WINDOW_BUTTON_CLOSE_Y,
                "CloseButton/default/pressed",
                1.0,
                0.0,
                None
            );
        }

        if close_response.clicked() {
            self.enabled = false;
        }
    }

    fn draw_trims(&mut self, ui: &mut egui::Ui, trims: u8) {
        ControlPanelHelpers::draw_combobox(
            ui,
            ControlPanelSettingsDimensions::TRIM_LINE_OUT_X,
            ControlPanelSettingsDimensions::TRIM_Y,
            ControlPanelSettingsDimensions::TRIM_WIDTH,
            (trims & 0x38) >> 3,
            &Self::LINE_OUT_ADC_TRIM_SELECTIONS,
            |selected| {
                self.orion_hid
                    .lock()
                    .unwrap()
                    .queue_trim(selected * 0x04 + 1);
            },
        );

        ControlPanelHelpers::draw_combobox(
            ui,
            ControlPanelSettingsDimensions::TRIM_ADC_X,
            ControlPanelSettingsDimensions::TRIM_Y,
            ControlPanelSettingsDimensions::TRIM_WIDTH,
            trims & 0x07,
            &Self::LINE_OUT_ADC_TRIM_SELECTIONS,
            |selected| {
                self.orion_hid
                    .lock()
                    .unwrap()
                    .queue_trim(selected * 0x04);
            },
        );
    }

    fn draw_oscillators(&mut self, ui: &mut egui::Ui, oscillators: u8) {
        let osc_mute_1_response = if oscillators & 0x40 != 0 {
            render_asset!(
                ui,
                ControlPanelSettingsDimensions::OSCILLATOR_MUTE_1_X,
                ControlPanelSettingsDimensions::OSCILLATOR_MUTE_Y,
                "SettingsPanelButton/default/pressed",
                1.0,
                0.0,
                Some(egui::Sense::click())
            )
        } else {
            render_asset!(
                ui,
                ControlPanelSettingsDimensions::OSCILLATOR_MUTE_1_X,
                ControlPanelSettingsDimensions::OSCILLATOR_MUTE_Y,
                "SettingsPanelButton/default/released",
                1.0,
                0.0,
                Some(egui::Sense::click())
            )
        };

        if ControlPanelHelpers::mouse_down_on(&osc_mute_1_response) {
            self.orion_hid
                .lock()
                .unwrap()
                .queue_oscillators(oscillators ^ 0x40);
        }

        ControlPanelHelpers::draw_combobox(
            ui,
            ControlPanelSettingsDimensions::OSCILLATOR_FREQUENCY_1_X,
            ControlPanelSettingsDimensions::OSCILLATOR_Y,
            ControlPanelSettingsDimensions::OSCILLATOR_WIDTH,
            (oscillators & 0x01 != 0) as u8,
            &Self::OSC_FREQUENCY_SELECTIONS,
            |selected| {
                let oscillators_new = (oscillators & !0x01) | (selected * 0x01);

                self.orion_hid
                    .lock()
                    .unwrap()
                    .queue_oscillators(oscillators_new);
            },
        );

        ControlPanelHelpers::draw_combobox(
            ui,
            ControlPanelSettingsDimensions::OSCILLATOR_LEVEL_X,
            ControlPanelSettingsDimensions::OSCILLATOR_Y,
            ControlPanelSettingsDimensions::OSCILLATOR_WIDTH,
            (oscillators & 0x30) >> 4,
            &Self::OSC_LEVEL_SELECTIONS,
            |selected| {
                let oscillators_new = (oscillators & !0x30) | (selected << 4);

                self.orion_hid
                    .lock()
                    .unwrap()
                    .queue_oscillators(oscillators_new);
            },
        );

        ControlPanelHelpers::draw_combobox(
            ui,
            ControlPanelSettingsDimensions::OSCILLATOR_FREQUENCY_2_X,
            ControlPanelSettingsDimensions::OSCILLATOR_Y,
            ControlPanelSettingsDimensions::OSCILLATOR_WIDTH,
            (oscillators & 0x04 != 0) as u8,
            &Self::OSC_FREQUENCY_SELECTIONS,
            |selected| {
                let oscillators_new = (oscillators & !0x04) | (selected * 0x04);

                self.orion_hid
                    .lock()
                    .unwrap()
                    .queue_oscillators(oscillators_new);
            },
        );

        let osc_mute_2_response = if oscillators & 0x80 != 0 {
            render_asset!(
                ui,
                ControlPanelSettingsDimensions::OSCILLATOR_MUTE_2_X,
                ControlPanelSettingsDimensions::OSCILLATOR_MUTE_Y,
                "SettingsPanelButton/default/pressed",
                1.0,
                0.0,
                Some(egui::Sense::click())
            )
        } else {
            render_asset!(
                ui,
                ControlPanelSettingsDimensions::OSCILLATOR_MUTE_2_X,
                ControlPanelSettingsDimensions::OSCILLATOR_MUTE_Y,
                "SettingsPanelButton/default/released",
                1.0,
                0.0,
                Some(egui::Sense::click())
            )
        };

        if ControlPanelHelpers::mouse_down_on(&osc_mute_2_response) {
            self.orion_hid
                .lock()
                .unwrap()
                .queue_oscillators(oscillators ^ 0x80);
        }
    }

    fn draw_brightness(&mut self, ui: &mut egui::Ui, brightness: u8) {
        let hitbox_rect = egui::Rect::from_min_size(
            egui::pos2(
                ControlPanelSettingsDimensions::BRIGHTNESS_SLIDER_HITBOX_X,
                ControlPanelSettingsDimensions::BRIGHTNESS_SLIDER_HITBOX_Y,
            ),
            egui::vec2(
                ControlPanelSettingsDimensions::BRIGHTNESS_SLIDER_HITBOX_WIDTH,
                ControlPanelSettingsDimensions::BRIGHTNESS_SLIDER_HITBOX_HEIGHT,
            ),
        );

        let (brightness, should_write) = self.drag_value.lock().unwrap().value_with_drag(
            ui,
            hitbox_rect,
            brightness,
            0x00,
            0xff,
            ControlPanelSettingsDimensions::BRIGHTNESS_SLIDER_TRAVEL,
            -1.0,
            true,
        );

        if should_write {
            self.orion_hid.lock().unwrap().queue_brightness(brightness);
        }

        let slider_pos_x = ControlPanelSettingsDimensions::BRIGHTNESS_SLIDER_X
            + brightness as u16 * ControlPanelSettingsDimensions::BRIGHTNESS_SLIDER_TRAVEL / 0xff;

        render_asset!(
            ui,
            slider_pos_x,
            ControlPanelSettingsDimensions::BRIGHTNESS_SLIDER_Y,
            "BrightnessSlider/default/layers/handle",
            1.0,
            0.0,
            None
        );

        let brightness_text_hitbox_rect = egui::Rect::from_min_size(
            egui::pos2(
                ControlPanelSettingsDimensions::BRIGHTNESS_TEXT_HITBOX_X,
                ControlPanelSettingsDimensions::BRIGHTNESS_TEXT_HITBOX_Y,
            ),
            egui::vec2(
                ControlPanelSettingsDimensions::BRIGHTNESS_TEXT_HITBOX_WIDTH,
                ControlPanelSettingsDimensions::BRIGHTNESS_TEXT_HITBOX_HEIGHT,
            ),
        );

        let brightness_text = (brightness as f32 * 100.0 / 255.0).round().to_string();

        if let Some((text, tab)) = self.text_editor.lock().unwrap().draw(
            ui,
            brightness_text_hitbox_rect,
            "brightness",
            &brightness_text,
            Self::FONT_SIZE,
            Self::FONT_COLOR,
        ) {
            if tab.is_none() {
                if let Some(brightness_new) = Self::brightness_for_text(&text) {
                    self.orion_hid
                        .lock()
                        .unwrap()
                        .queue_brightness(brightness_new);
                }
            }
        }
    }

    fn draw_madi_smux(&mut self, ui: &mut egui::Ui, madi_smux: u8) {
        let madi_in_response = if madi_smux & 0x40 != 0 {
            render_asset!(
                ui,
                ControlPanelSettingsDimensions::MADI_SMUX_IN_X,
                ControlPanelSettingsDimensions::MADI_SMUX_Y,
                "SettingsPanelButton/default/pressed",
                1.0,
                0.0,
                Some(egui::Sense::click())
            )
        } else {
            render_asset!(
                ui,
                ControlPanelSettingsDimensions::MADI_SMUX_IN_X,
                ControlPanelSettingsDimensions::MADI_SMUX_Y,
                "SettingsPanelButton/default/released",
                1.0,
                0.0,
                Some(egui::Sense::click())
            )
        };

        if ControlPanelHelpers::mouse_down_on(&madi_in_response) {
            let madi_smux_new = (madi_smux >> 6) ^ 0x01;

            self.orion_hid
                .lock()
                .unwrap()
                .queue_madi_smux(madi_smux_new);
        }

        let madi_out_response = if madi_smux & 0x80 != 0 {
            render_asset!(
                ui,
                ControlPanelSettingsDimensions::MADI_SMUX_OUT_X,
                ControlPanelSettingsDimensions::MADI_SMUX_Y,
                "SettingsPanelButton/default/pressed",
                1.0,
                0.0,
                Some(egui::Sense::click())
            )
        } else {
            render_asset!(
                ui,
                ControlPanelSettingsDimensions::MADI_SMUX_OUT_X,
                ControlPanelSettingsDimensions::MADI_SMUX_Y,
                "SettingsPanelButton/default/released",
                1.0,
                0.0,
                Some(egui::Sense::click())
            )
        };

        if ControlPanelHelpers::mouse_down_on(&madi_out_response) {
            let madi_smux_new = (madi_smux >> 6) ^ 0x02;

            self.orion_hid
                .lock()
                .unwrap()
                .queue_madi_smux(madi_smux_new);
        }
    }

    fn brightness_for_text(text: &String) -> Option<u8> {
        let text_parsed = text.trim();

        let text_parsed = match text_parsed.parse::<u16>() {
            Ok(value) => value,
            Err(_) => return None,
        };

        if text_parsed > 100 {
            return None;
        }

        Some((text_parsed * 0xff / 100) as u8)
    }
}
