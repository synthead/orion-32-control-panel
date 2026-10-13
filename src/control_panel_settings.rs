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
                self.draw_base_graphics(ui);
                self.draw_window_buttons(ui);
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
