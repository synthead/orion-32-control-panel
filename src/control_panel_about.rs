use std::sync::Arc;
use std::sync::Mutex;

use crate::constants::WidgetColors;

use crate::orion_state::{OrionState, OrionStates};

use crate::render_asset;

struct ControlPanelAboutDimensions;

impl ControlPanelAboutDimensions {
    const ABOUT_X: f32 = 404.0;
    const ABOUT_Y: f32 = 265.0;

    const TEXT_X: f32 = 593.0;
    const TEXT_CONTROL_PANEL_VERISON_Y: f32 = 290.0;
    const TEXT_HARDWARE_VERSION_Y: f32 = 322.0;
    const TEXT_SERIAL_NUMBER_Y: f32 = 355.0;
    const TEXT_FIRMWARE_VERSION_Y: f32 = 387.0;

    const TEXT_WIDTH: f32 = 141.0;
    const TEXT_HEIGHT: f32 = 22.0;

    const WINDOW_BUTTON_CLOSE_X: u16 = 744;
    const WINDOW_BUTTON_CLOSE_Y: u16 = 273;
}

pub(crate) struct ControlPanelAbout {
    orion_states: Arc<Mutex<OrionStates>>,

    pub(crate) enabled: bool,
}

impl ControlPanelAbout {
    pub(crate) fn new(orion_states: Arc<Mutex<OrionStates>>) -> Self {
        Self {
            orion_states,

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
        self.draw_fields(ui, &orion_state);
    }

    fn draw_base_graphics(&mut self, ui: &mut egui::Ui) {
        render_asset!(
            ui,
            ControlPanelAboutDimensions::ABOUT_X,
            ControlPanelAboutDimensions::ABOUT_Y,
            "AboutBox/default/background-edited",
            1.0,
            0.0,
            None
        );
    }

    fn draw_window_buttons(&mut self, ui: &mut egui::Ui) {
        let close_response = render_asset!(
            ui,
            ControlPanelAboutDimensions::WINDOW_BUTTON_CLOSE_X,
            ControlPanelAboutDimensions::WINDOW_BUTTON_CLOSE_Y,
            "CloseButton/default/released",
            1.0,
            0.0,
            Some(egui::Sense::click())
        );

        if close_response.is_pointer_button_down_on() != close_response.contains_pointer() {
            render_asset!(
                ui,
                ControlPanelAboutDimensions::WINDOW_BUTTON_CLOSE_X,
                ControlPanelAboutDimensions::WINDOW_BUTTON_CLOSE_Y,
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

    fn draw_fields(&mut self, ui: &mut egui::Ui, orion_state: &OrionState) {
        let control_panel_version = format!("OSS {}", env!("CARGO_PKG_VERSION"));
        let hardware_version = String::from_utf8_lossy(&orion_state.hardware_version).to_string();
        let serial = String::from_utf8_lossy(&orion_state.serial).to_string();
        let firmware_version =
            (u16::from_be_bytes(orion_state.firmware_version) as f32 / 100.0).to_string();

        self.draw_field(
            ui,
            control_panel_version,
            ControlPanelAboutDimensions::TEXT_CONTROL_PANEL_VERISON_Y,
        );

        self.draw_field(
            ui,
            hardware_version,
            ControlPanelAboutDimensions::TEXT_HARDWARE_VERSION_Y,
        );

        self.draw_field(
            ui,
            serial,
            ControlPanelAboutDimensions::TEXT_SERIAL_NUMBER_Y,
        );

        self.draw_field(
            ui,
            firmware_version,
            ControlPanelAboutDimensions::TEXT_FIRMWARE_VERSION_Y,
        );
    }

    fn draw_field(&mut self, ui: &mut egui::Ui, text: String, pos_y: f32) {
        let rect = egui::Rect::from_min_size(
            egui::pos2(ControlPanelAboutDimensions::TEXT_X, pos_y),
            egui::vec2(
                ControlPanelAboutDimensions::TEXT_WIDTH,
                ControlPanelAboutDimensions::TEXT_HEIGHT,
            ),
        );

        let label =
            egui::Label::new(egui::RichText::new(text).color(WidgetColors::TEXT_COLOR_ABOUT));

        ui.put(rect, label.selectable(true));
    }
}
