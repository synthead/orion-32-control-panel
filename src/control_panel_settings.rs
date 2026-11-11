use std::sync::Arc;
use std::sync::Mutex;

use crate::constants::{WidgetColors, WidgetDimensions};

use crate::orion_hid::OrionHid;
use crate::orion_state::OrionStates;

use crate::drag_value::DragValue;
use crate::ignore_value::IgnoreValue;
use crate::text_editor::TextEditor;

use crate::control_panel_helpers::ControlPanelHelpers;

use crate::render_asset;

struct ControlPanelSettingsDimensions;

impl ControlPanelSettingsDimensions {
    const SETTINGS_X: f32 = 286.0;
    const SETTINGS_Y: f32 = 179.0;

    const SETTINGS_LABEL_X: u16 = 816;
    const SETTINGS_LABEL_Y: u16 = 187;

    const WINDOW_BUTTON_CLOSE_X: u16 = 885;
    const WINDOW_BUTTON_CLOSE_Y: u16 = 185;

    const TRIM_LINE_OUT_X: u16 = 461;
    const TRIM_ADC_X: u16 = 598;
    const TRIM_Y: u16 = 265;
    const TRIM_WIDTH: u8 = 68;

    const ACTIVATE_X: u16 = 738;
    const ACTIVATE_Y: u16 = 266;

    const OSCILLATOR_MUTE_1_X: u16 = 347;
    const OSCILLATOR_MUTE_2_X: u16 = 836;
    const OSCILLATOR_MUTE_Y: u16 = 352;

    const OSCILLATOR_FREQUENCY_1_X: u16 = 431;
    const OSCILLATOR_LEVEL_X: u16 = 568;
    const OSCILLATOR_FREQUENCY_2_X: u16 = 707;
    const OSCILLATOR_Y: u16 = 351;
    const OSCILLATOR_WIDTH: u8 = 68;

    const BRIGHTNESS_SLIDER_X: u16 = 306;
    const BRIGHTNESS_SLIDER_Y: u16 = 433;
    const BRIGHTNESS_SLIDER_TRAVEL: u16 = 69;

    const BRIGHTNESS_SLIDER_HITBOX_X: f32 = 306.0;
    const BRIGHTNESS_SLIDER_HITBOX_Y: f32 = 433.0;
    const BRIGHTNESS_SLIDER_HITBOX_WIDTH: f32 = 112.0;
    const BRIGHTNESS_SLIDER_HITBOX_HEIGHT: f32 = 21.0;

    const BRIGHTNESS_TEXT_HITBOX_X: f32 = 429.0;
    const BRIGHTNESS_TEXT_HITBOX_Y: f32 = 434.0;
    const BRIGHTNESS_TEXT_HITBOX_WIDTH: f32 = 52.0;
    const BRIGHTNESS_TEXT_HITBOX_HEIGHT: f32 = 18.0;

    const USB_CHANNELS_X: u16 = 562;
    const USB_CHANNELS_Y: u16 = 435;
    const USB_CHANNELS_WIDTH: u8 = 52;

    const MADI_SMUX_IN_X: u16 = 684;
    const MADI_SMUX_OUT_X: u16 = 781;
    const MADI_SMUX_Y: u16 = 436;

    const SPDIF_SRC_X: u16 = 858;
    const SPDIF_SRC_Y: u16 = 436;
}

pub(crate) struct ControlPanelSettings {
    orion_states: Arc<Mutex<OrionStates>>,
    orion_hid: Arc<Mutex<OrionHid>>,

    text_editor: Arc<Mutex<TextEditor>>,
    drag_value: Arc<Mutex<DragValue>>,

    brightness_ignore: IgnoreValue,
    pub(crate) enabled: bool,
}

impl ControlPanelSettings {
    const OSC_FREQUENCY_SELECTIONS: [&str; 2] = ["1 kHz", "440 Hz"];

    const OSC_LEVEL_SELECTIONS: [&str; 4] = ["0 dBFS", "-6 dbFS", "-12 dbFS", "-18 dbFS"];

    const LINE_OUT_ADC_TRIM_SELECTIONS: [&str; 7] = [
        "20 dBu", "19 dBu", "18 dBu", "17 dBu", "16 dBu", "15 dBu", "14 dBu",
    ];

    const USB_CHANNELS_SELECTIONS: [&str; 2] = ["24", "32"];

    pub(crate) fn new(
        orion_states: Arc<Mutex<OrionStates>>,
        orion_hid: Arc<Mutex<OrionHid>>,
        text_editor: Arc<Mutex<TextEditor>>,
        drag_value: Arc<Mutex<DragValue>>,
    ) -> Self {
        Self {
            orion_states,
            orion_hid,

            text_editor,
            drag_value,

            brightness_ignore: IgnoreValue::new(),
            enabled: false,
        }
    }

    pub(crate) fn draw(&mut self, ui: &mut egui::Ui) {
        if !self.enabled {
            return;
        }

        let Some(orion_state) = self.orion_states.lock().unwrap().current().cloned() else {
            return;
        };

        self.draw_base_graphics(ui);
        self.draw_window_buttons(ui);

        self.draw_trims(ui, orion_state.trims);
        self.draw_activate(ui, orion_state.activate);
        self.draw_oscillators(ui, orion_state.oscillators);
        self.draw_brightness(ui, orion_state.brightness);
        self.draw_madi_smux(ui, orion_state.madi_smux);
        self.draw_usb_channels_spdif_src(ui, orion_state.usb_channels_spdif_src);
    }

    fn draw_base_graphics(&mut self, ui: &mut egui::Ui) {
        render_asset!(
            ui,
            ControlPanelSettingsDimensions::SETTINGS_X,
            ControlPanelSettingsDimensions::SETTINGS_Y,
            "SettingsPanel/default/background-edited",
            1.0,
            0.0,
            None
        );

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

        if close_response.is_pointer_button_down_on() != close_response.contains_pointer() {
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
            None,
            &Self::LINE_OUT_ADC_TRIM_SELECTIONS,
            true,
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
            None,
            &Self::LINE_OUT_ADC_TRIM_SELECTIONS,
            true,
            |selected| {
                self.orion_hid.lock().unwrap().queue_trim(selected * 0x04);
            },
        );
    }

    fn draw_activate(&mut self, ui: &mut egui::Ui, activate: u8) {
        let activate_response = if activate == 0x01 {
            render_asset!(
                ui,
                ControlPanelSettingsDimensions::ACTIVATE_X,
                ControlPanelSettingsDimensions::ACTIVATE_Y,
                "SettingsPanelButton/default/pressed",
                1.0,
                0.0,
                Some(egui::Sense::click())
            )
        } else {
            render_asset!(
                ui,
                ControlPanelSettingsDimensions::ACTIVATE_X,
                ControlPanelSettingsDimensions::ACTIVATE_Y,
                "SettingsPanelButton/default/released",
                1.0,
                0.0,
                Some(egui::Sense::click())
            )
        };

        if ControlPanelHelpers::mouse_down_on(&activate_response) {
            self.orion_hid
                .lock()
                .unwrap()
                .queue_activate(1 - activate);
        }
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
            None,
            &Self::OSC_FREQUENCY_SELECTIONS,
            true,
            |selected| {
                let oscillators_new = (oscillators & !0x01) | selected;

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
            None,
            &Self::OSC_LEVEL_SELECTIONS,
            true,
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
            None,
            &Self::OSC_FREQUENCY_SELECTIONS,
            true,
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

    fn draw_brightness(&mut self, ui: &mut egui::Ui, mut brightness: u8) {
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

        brightness = self.brightness_ignore.value_with_ignore(brightness);

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
            self.brightness_ignore.begin_ignore(brightness);
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
            egui::Id::new("brightness"),
            &brightness_text,
            WidgetDimensions::TEXT_SIZE,
            WidgetColors::TEXT_COLOR,
        ) && tab.is_none()
            && let Some(brightness_new) = Self::brightness_for_text(&text)
        {
            self.orion_hid
                .lock()
                .unwrap()
                .queue_brightness(brightness_new);
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

    fn draw_usb_channels_spdif_src(&mut self, ui: &mut egui::Ui, usb_channels_spdif_src: u8) {
        ControlPanelHelpers::draw_combobox(
            ui,
            ControlPanelSettingsDimensions::USB_CHANNELS_X,
            ControlPanelSettingsDimensions::USB_CHANNELS_Y,
            ControlPanelSettingsDimensions::USB_CHANNELS_WIDTH,
            (usb_channels_spdif_src & 0x02) >> 1,
            None,
            &Self::USB_CHANNELS_SELECTIONS,
            true,
            |selected| {
                self.orion_hid.lock().unwrap().queue_usb_channels(selected);
            },
        );

        let spdif_src_response = if usb_channels_spdif_src & 0x01 != 0 {
            render_asset!(
                ui,
                ControlPanelSettingsDimensions::SPDIF_SRC_X,
                ControlPanelSettingsDimensions::SPDIF_SRC_Y,
                "SettingsPanelButton/default/pressed",
                1.0,
                0.0,
                Some(egui::Sense::click())
            )
        } else {
            render_asset!(
                ui,
                ControlPanelSettingsDimensions::SPDIF_SRC_X,
                ControlPanelSettingsDimensions::SPDIF_SRC_Y,
                "SettingsPanelButton/default/released",
                1.0,
                0.0,
                Some(egui::Sense::click())
            )
        };

        if ControlPanelHelpers::mouse_down_on(&spdif_src_response) {
            self.orion_hid
                .lock()
                .unwrap()
                .queue_spdif_src(usb_channels_spdif_src ^ 0x01);
        }
    }

    fn brightness_for_text(text: &str) -> Option<u8> {
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
