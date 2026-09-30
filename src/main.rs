// Hide console window on Windows.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;
use eframe::egui::Widget;

struct Orion32ControlPanel;

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

impl eframe::App for Orion32ControlPanel {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ui, |ui| {
                egui::Image::new(egui::include_image!(
                    "../assets/ControlPanel/default/background.png"
                ))
                .show_loading_spinner(false)
                .ui(ui);

                ui.put(
                    egui::Rect::from_min_size(egui::pos2(500.0, 500.0), egui::vec2(100.0, 20.0)),
                    egui::Label::new("Hello World!"),
                );
            });
    }
}
