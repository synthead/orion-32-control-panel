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

    fn ui_content(&mut self, ui: &mut egui::Ui) {
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ui, |ui| {
                self.draw_base_graphics(ui);
            });
    }

    fn draw_base_graphics(&mut self, ui: &mut egui::Ui) {
        render_asset!(ui, 0, 0, "SettingsPanel/default/background", 1.0, 0.0, None);
    }
}
