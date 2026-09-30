// Hide console window on Windows.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;
use eframe::egui::Widget;

struct Orion32ControlPanel;

impl Orion32ControlPanel {
    fn ui_content(&mut self, ui: &mut egui::Ui) {
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ui, |ui| {
                egui::Image::new(egui::include_image!(
                    "../assets/ControlPanel/default/background.png"
                ))
                .show_loading_spinner(false)
                .ui(ui);

                ui.put(
                    egui::Rect::from_min_size(egui::pos2(0.0, 226.0), egui::vec2(1180.0, 453.0)),
                    egui::Image::new(egui::include_image!(
                        "../assets/Routing/default/background.png"
                    ))
                    .show_loading_spinner(false),
                );

                ui.put(
                    egui::Rect::from_min_size(egui::pos2(0.0, 226.0), egui::vec2(1180.0, 453.0)),
                    egui::Image::new(egui::include_image!(
                        "../assets/Routing/default/background.png"
                    ))
                    .show_loading_spinner(false),
                );

                ui.put(
                    egui::Rect::from_min_size(egui::pos2(20.0, 8.0), egui::vec2(17.0, 17.0)),
                    egui::Image::new(egui::include_image!(
                        "../assets/StandbyButton/default/pressed.png"
                    ))
                    .show_loading_spinner(false),
                );

                ui.put(
                    egui::Rect::from_min_size(egui::pos2(1075.0, 9.0), egui::vec2(14.0, 14.0)),
                    egui::Image::new(egui::include_image!(
                        "../assets/SettingsButton/default/released.png"
                    ))
                    .show_loading_spinner(false),
                );

                ui.put(
                    egui::Rect::from_min_size(egui::pos2(1124.0, 9.0), egui::vec2(14.0, 14.0)),
                    egui::Image::new(egui::include_image!(
                        "../assets/MinimizeButton/default/released.png"
                    ))
                    .show_loading_spinner(false),
                );

                ui.put(
                    egui::Rect::from_min_size(egui::pos2(1100.0, 9.0), egui::vec2(14.0, 14.0)),
                    egui::Image::new(egui::include_image!(
                        "../assets/AboutButton/default/released.png"
                    ))
                    .show_loading_spinner(false),
                );

                ui.put(
                    egui::Rect::from_min_size(egui::pos2(1149.0, 9.0), egui::vec2(14.0, 14.0)),
                    egui::Image::new(egui::include_image!(
                        "../assets/CloseButton/default/released.png"
                    ))
                    .show_loading_spinner(false),
                );

                ui.put(
                    egui::Rect::from_min_size(egui::pos2(500.0, 500.0), egui::vec2(100.0, 20.0)),
                    egui::Label::new("Hello World!"),
                );
            });
    }
}

impl eframe::App for Orion32ControlPanel {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.ui_content(ui);
    }
}

fn main() -> eframe::Result {
    // Log to stderr with RUST_LOG=debug env var.
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1180.0, 700.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Orion 32 Control Panel",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);

            Ok(Box::new(Orion32ControlPanel))
        }),
    )
}
