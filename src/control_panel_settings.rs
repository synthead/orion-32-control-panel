use std::sync::Arc;
use std::sync::Mutex;

use crate::orion_hid::OrionHid;
use crate::orion_state::OrionState;

use crate::drag_value::DragValue;
use crate::text_editor::TextEditor;

use crate::render_asset;

struct ControlPanelSettingsDimensions;

impl ControlPanelSettingsDimensions {
    const VIEWPORT_X: f32 = 621.0;
    const VIEWPORT_Y: f32 = 392.0;

    const SETTINGS_LABEL_X: u16 = 530;
    const SETTINGS_LABEL_Y: u16 = 8;

    const WINDOW_BUTTON_CLOSE_X: u16 = 599;
    const WINDOW_BUTTON_CLOSE_Y: u16 = 6;

    const BRIGHTNESS_SLIDER_X: u16 = 20;
    const BRIGHTNESS_SLIDER_Y: u16 = 254;
    const BRIGHTNESS_SLIDER_TRAVEL: u16 = 69;

    const BRIGHTNESS_SLIDER_HITBOX_X: f32 = 20.0;
    const BRIGHTNESS_SLIDER_HITBOX_Y: f32 = 254.0;
    const BRIGHTNESS_SLIDER_HITBOX_WIDTH: f32 = 112.0;
    const BRIGHTNESS_SLIDER_HITBOX_HEIGHT: f32 = 21.0;
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
                self.draw_brightness(ui, orion_state.brightness);
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

        let (level, should_write) = self.drag_value.lock().unwrap().value_with_drag(
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
            // TODO: Write brightness.
        }

        let slider_pos_x = ControlPanelSettingsDimensions::BRIGHTNESS_SLIDER_X
            + level as u16 * ControlPanelSettingsDimensions::BRIGHTNESS_SLIDER_TRAVEL / 0xff;

        render_asset!(
            ui,
            slider_pos_x,
            ControlPanelSettingsDimensions::BRIGHTNESS_SLIDER_Y,
            "BrightnessSlider/default/layers/handle",
            1.0,
            0.0,
            None
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
}
