// Hide console window on Windows.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;

struct Orion32ControlPanel;

macro_rules! render_asset {
    ($ui:expr, $x:expr, $y:expr, $subpath:literal) => {
        $ui.scope_builder(
            egui::UiBuilder::new().max_rect(egui::Rect::from_min_size(
                egui::pos2($x, $y),
                $ui.available_size(),
            )),
            |ui| {
                ui.add(
                    egui::Image::new(egui::include_image!(concat!(
                        "../assets/",
                        $subpath,
                        ".png"
                    )))
                    .fit_to_original_size(1.0)
                    .show_loading_spinner(false),
                );
            },
        );
    };
}

impl Orion32ControlPanel {
    fn ui_content(&mut self, ui: &mut egui::Ui) {
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ui, |ui| {
                render_asset!(ui, 0.0, 0.0, "ControlPanel/default/background");

                render_asset!(ui, 0.0, 226.0, "Routing/default/background");
                render_asset!(ui, 20.0, 8.0, "StandbyButton/default/pressed");
                render_asset!(ui, 1075.0, 9.0, "SettingsButton/default/released");
                render_asset!(ui, 1124.0, 9.0, "MinimizeButton/default/released");
                render_asset!(ui, 1100.0, 9.0, "AboutButton/default/released");
                render_asset!(ui, 1149.0, 9.0, "CloseButton/default/released");

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
