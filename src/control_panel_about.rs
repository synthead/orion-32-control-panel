use std::sync::Arc;
use std::sync::Mutex;

use crate::orion_state::OrionState;

use crate::control_panel_helpers::ControlPanelHelpers;

use crate::render_asset;

struct ControlPanelAboutDimensions;

impl ControlPanelAboutDimensions {
    const ABOUT_X: f32 = 404.0;
    const ABOUT_Y: f32 = 243.0;

    const WINDOW_BUTTON_CLOSE_X: u16 = 744;
    const WINDOW_BUTTON_CLOSE_Y: u16 = 251;
}

pub(crate) struct ControlPanelAbout {
    orion_state: Arc<Mutex<OrionState>>,

    pub(crate) enabled: bool,
}

impl ControlPanelAbout {
    const FONT_COLOR: egui::Color32 = egui::Color32::from_rgb(0x95, 0x95, 0x95);
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
}
