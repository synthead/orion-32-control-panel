use std::sync::Arc;
use std::sync::Mutex;

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

    const BUTTON_X: u16 = 504;
    const CONNECT_BUTTON_Y: u16 = 504;
    const ADMINISTRATION_BUTTON_Y: u16 = 554;
}

struct ControlPanelDevicesCoiors;

impl ControlPanelDevicesCoiors {
    const BORDER: egui::Color32 = egui::Color32::from_rgb(0x82, 0x87, 0x90);
    const TEXT: egui::Color32 = egui::Color32::WHITE;
}

pub(crate) struct ControlPanelDevices {
    orion_state: Arc<Mutex<OrionState>>,
}

impl ControlPanelDevices {
    const INIT_PACKETS: [&str; 6] = [
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
    }

    fn draw_base_graphics(ui: &mut egui::Ui) {
        let response = render_asset!(
            ui,
            0,
            0,
            "AvailableDevicesSlide/default/background",
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

        render_asset!(
            ui,
            ControlPanelDevicesDimensions::BUTTON_X,
            ControlPanelDevicesDimensions::CONNECT_BUTTON_Y,
            "ConnectButton/default/released",
            1.0,
            0.0,
            None
        );

        render_asset!(
            ui,
            ControlPanelDevicesDimensions::BUTTON_X,
            ControlPanelDevicesDimensions::ADMINISTRATION_BUTTON_Y,
            "AdministrationLargeButton/default/released",
            1.0,
            0.0,
            None
        );
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

        if close_response.is_pointer_button_down_on() != close_response.hovered() {
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

        egui::Area::new(egui::Id::new("hello_world"))
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
                    egui::Stroke::new(1.0, ControlPanelDevicesCoiors::BORDER),
                    egui::StrokeKind::Inside,
                );

                egui::Frame::new().inner_margin(3.0).show(ui, |ui| {
                    if inits_completed & OrionStateInitSteps::SERIAL_NUMBER != 0 {
                        ui.add(
                            egui::Label::new(
                                egui::RichText::new(serial_text)
                                    .color(ControlPanelDevicesCoiors::TEXT),
                            )
                            .selectable(false),
                        );
                    }
                });
            });
    }
}
