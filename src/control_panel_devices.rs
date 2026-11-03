use std::sync::Arc;
use std::sync::Mutex;

use crate::constants::{WidgetColors, WidgetDimensions};

use crate::orion_state::OrionState;
use crate::orion_state::OrionStateInitSteps;

use crate::render_asset;

struct ControlPanelDevicesDimensions;

impl ControlPanelDevicesDimensions {
    const WINDOW_BUTTON_CLOSE_X: u16 = 1149;
    const WINDOW_BUTTON_CLOSE_YY: u16 = 9;

    const DEVICES_LIST_X: f32 = 309.0;
    const DEVICES_LIST_Y: f32 = 272.0;
    const DEVICES_LIST_WIDTH: f32 = 560.0;
    const DEVICES_LIST_HEIGHT: f32 = 188.0;

    const STATUS_TEXT_X: f32 = 590.0;
    const STATUS_TEXT_Y: f32 = 525.0;
}

struct ControlPanelDevicesColors;

impl ControlPanelDevicesColors {
    const DEVIES_BORDER: egui::Color32 = egui::Color32::from_rgb(0x82, 0x87, 0x90);
}

pub(crate) struct ControlPanelDevices {
    orion_state: Arc<Mutex<OrionState>>,
}

impl ControlPanelDevices {
    const QUERYING_MESSAGES: [&str; 6] = [
        "serial number",
        "firmware version",
        "routes",
        "mixer levels",
        "mixer pans",
        "AD phases",
    ];

    pub(crate) fn new(orion_state: Arc<Mutex<OrionState>>) -> Self {
        Self { orion_state }
    }

    pub(crate) fn draw(&mut self, ui: &mut egui::Ui) {
        let orion_state = self.orion_state.lock().unwrap().clone();

        Self::draw_base_graphics(ui);
        self.draw_window_buttons(ui);

        self.draw_devices_list(ui, &orion_state.serial, orion_state.inits_completed);
        self.draw_text(ui, orion_state.inits_completed);
    }

    fn draw_base_graphics(ui: &mut egui::Ui) {
        let response = render_asset!(
            ui,
            0,
            0,
            "AvailableDevicesSlide/default/background-edited",
            1.0,
            0.0,
            Some(egui::Sense::click())
        );

        if !egui::Popup::is_any_open(ui.ctx())
            && response.is_pointer_button_down_on()
            && ui.input(|i| i.pointer.button_pressed(egui::PointerButton::Primary))
        {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
        }
    }

    fn draw_window_buttons(&mut self, ui: &mut egui::Ui) {
        let close_response = render_asset!(
            ui,
            ControlPanelDevicesDimensions::WINDOW_BUTTON_CLOSE_X,
            ControlPanelDevicesDimensions::WINDOW_BUTTON_CLOSE_YY,
            "CloseButton/default/released",
            1.0,
            0.0,
            Some(egui::Sense::click())
        );

        if close_response.is_pointer_button_down_on() != close_response.contains_pointer() {
            render_asset!(
                ui,
                ControlPanelDevicesDimensions::WINDOW_BUTTON_CLOSE_X,
                ControlPanelDevicesDimensions::WINDOW_BUTTON_CLOSE_YY,
                "CloseButton/default/pressed",
                1.0,
                0.0,
                None
            );
        }

        if close_response.clicked() {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }

    fn draw_devices_list(&mut self, ui: &mut egui::Ui, serial: &[u8; 13], inits_completed: u8) {
        let serial_text = String::from_utf8_lossy(serial);

        egui::Area::new(egui::Id::new("devices_list"))
            .fixed_pos(egui::pos2(
                ControlPanelDevicesDimensions::DEVICES_LIST_X,
                ControlPanelDevicesDimensions::DEVICES_LIST_Y,
            ))
            .show(ui.ctx(), |ui| {
                ui.set_min_size(egui::vec2(
                    ControlPanelDevicesDimensions::DEVICES_LIST_WIDTH,
                    ControlPanelDevicesDimensions::DEVICES_LIST_HEIGHT,
                ));

                ui.painter().rect_stroke(
                    egui::Rect::from_min_size(
                        egui::pos2(
                            ControlPanelDevicesDimensions::DEVICES_LIST_X,
                            ControlPanelDevicesDimensions::DEVICES_LIST_Y,
                        ),
                        egui::vec2(
                            ControlPanelDevicesDimensions::DEVICES_LIST_WIDTH,
                            ControlPanelDevicesDimensions::DEVICES_LIST_HEIGHT,
                        ),
                    ),
                    0.0,
                    egui::Stroke::new(1.0, ControlPanelDevicesColors::DEVIES_BORDER),
                    egui::StrokeKind::Inside,
                );

                egui::Frame::new().inner_margin(3.0).show(ui, |ui| {
                    if inits_completed & OrionStateInitSteps::SERIAL_NUMBER != 0 {
                        ui.add(
                            egui::Label::new(
                                egui::RichText::new(serial_text)
                                    .color(WidgetColors::TEXT_COLOR_DEVICE),
                            )
                            .selectable(false),
                        );
                    }
                });
            });
    }

    fn draw_text(&mut self, ui: &mut egui::Ui, inits_completed: u8) {
        let text = if inits_completed == 0 {
            "Not connected.".to_string()
        } else {
            let index = (u8::BITS - 1 - inits_completed.leading_zeros()) as usize;

            format!("Querying {}...", Self::QUERYING_MESSAGES[index])
        };

        ui.painter().text(
            egui::pos2(
                ControlPanelDevicesDimensions::STATUS_TEXT_X,
                ControlPanelDevicesDimensions::STATUS_TEXT_Y,
            ),
            egui::Align2::CENTER_TOP,
            text,
            egui::FontId::proportional(WidgetDimensions::TEXT_SIZE_DEVICE_STATUS),
            WidgetColors::TEXT_COLOR_DEVICE_STATUS,
        );
    }
}
