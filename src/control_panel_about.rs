use std::sync::Arc;
use std::sync::Mutex;

use crate::orion_state::OrionState;

use crate::control_panel_helpers::ControlPanelHelpers;

use crate::render_asset;

struct ControlPanelAboutDimensions;

impl ControlPanelAboutDimensions {
    const ABOUT_X: f32 = 404.0;
    const ABOUT_Y: f32 = 243.0;

    const TEXT_X: f32 = 593.0;
    const TEXT_Y: f32 = 268.0;
    const TEXT_Y_STEP: f32 = 33.0;

    const TEXT_WIDTH: f32 = 141.0;
    const TEXT_HEIGHT: f32 = 22.0;

    const WINDOW_BUTTON_CLOSE_X: u16 = 744;
    const WINDOW_BUTTON_CLOSE_Y: u16 = 251;
}

pub(crate) struct ControlPanelAbout {
    orion_state: Arc<Mutex<OrionState>>,

    pub(crate) enabled: bool,
}

impl ControlPanelAbout {
    const FONT_COLOR: egui::Color32 = egui::Color32::WHITE;
    const FONT_SIZE: f32 = 10.5;

    pub(crate) fn new(orion_state: Arc<Mutex<OrionState>>) -> Self {
        Self {
            orion_state,

            enabled: false,
        }
    }

    pub(crate) fn draw(&mut self, ui: &mut egui::Ui) {
        if !self.enabled {
            return;
        }

        let orion_state = self.orion_state.lock().unwrap().clone();

        self.draw_base_graphics(ui);
        self.draw_window_buttons(ui);
        self.draw_fields(ui, &orion_state);
    }

    fn draw_base_graphics(&mut self, ui: &mut egui::Ui) {
        render_asset!(
            ui,
            ControlPanelAboutDimensions::ABOUT_X,
            ControlPanelAboutDimensions::ABOUT_Y,
            "AboutBox/default/background",
            1.0,
            0.0,
            Some(egui::Sense::click())
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

        if close_response.is_pointer_button_down_on() != close_response.hovered() {
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
        let control_panel_version = env!("CARGO_PKG_VERSION").to_string();
        let serial = String::from_utf8_lossy(&orion_state.serial).to_string();

        self.draw_field(ui, control_panel_version, 0);
        self.draw_field(ui, "test2".to_string(), 1);
        self.draw_field(ui, serial, 2);
        self.draw_field(ui, "test4".to_string(), 3);
    }

    fn draw_field(&mut self, ui: &mut egui::Ui, text: String, field: u8) {
        let pos_y = ControlPanelAboutDimensions::TEXT_Y
            + ControlPanelAboutDimensions::TEXT_Y_STEP * field as f32;

        let rect = egui::Rect::from_min_size(
            egui::pos2(ControlPanelAboutDimensions::TEXT_X, pos_y),
            egui::vec2(
                ControlPanelAboutDimensions::TEXT_WIDTH,
                ControlPanelAboutDimensions::TEXT_HEIGHT,
            ),
        );

        let label = egui::Label::new(egui::RichText::new(text).color(egui::Color32::WHITE));

        ui.put(rect, label.selectable(true));
    }
}
