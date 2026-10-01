// Hide console window on Windows.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;
use hidapi::HidApi;
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Default)]
struct Orion32Parameters {
    vu_left: [u8; 32],
    vu_right: [u8; 32],
}

struct Orion32ControlPanel {
    parameters: Arc<Mutex<Orion32Parameters>>,
}

macro_rules! render_asset {
    ($ui:expr, $x:expr, $y:expr, $subpath:literal) => {
        $ui.scope_builder(
            egui::UiBuilder::new().max_rect(egui::Rect::from_min_size(
                egui::pos2($x as f32, $y as f32),
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

macro_rules! render_peak {
    ($ui:expr, $x:expr, $y:expr, $db:expr, $subpath:literal) => {{
        let image = egui::Image::new(egui::include_image!(concat!(
            "../assets/",
            $subpath,
            ".png"
        )))
        .fit_to_original_size(1.0)
        .show_loading_spinner(false);

        if let Some(image_size) = image.load_and_calc_size($ui, $ui.available_size()) {
            let image_rect =
                egui::Rect::from_min_size(egui::pos2($x as f32, $y as f32), image_size);

            let clip_rect = egui::Rect::from_min_max(
                egui::pos2($x as f32, $y as f32 + image_size.y * $db as f32 / 96.0),
                image_rect.max,
            );

            $ui.scope_builder(egui::UiBuilder::new().max_rect(image_rect), |ui| {
                ui.set_clip_rect(clip_rect);
                ui.add(image);
            });
        }
    }};
}

impl Orion32ControlPanel {
    fn ui_content(&mut self, ui: &mut egui::Ui) {
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ui, |ui| {
                render_asset!(ui, 0, 0, "ControlPanel/default/background");

                render_asset!(ui, 0, 226, "Routing/default/background");
                render_asset!(ui, 20, 8, "StandbyButton/default/pressed");
                render_asset!(ui, 1075, 9, "SettingsButton/default/released");
                render_asset!(ui, 1124, 9, "MinimizeButton/default/released");
                render_asset!(ui, 1100, 9, "AboutButton/default/released");
                render_asset!(ui, 1149, 9, "CloseButton/default/released");

                let parameters = self.parameters.lock().unwrap();

                for peak_x in 0..32 {
                    let pos_x_1 = 135 + peak_x * 11;
                    let pos_x_2 = pos_x_1 + 581;

                    render_peak!(
                        ui,
                        pos_x_1,
                        60,
                        parameters.vu_left[peak_x],
                        "MonitorPeakMeter/default/background"
                    );
                    render_peak!(
                        ui,
                        pos_x_2,
                        60,
                        parameters.vu_right[peak_x],
                        "MonitorPeakMeter/default/background"
                    );
                }

                for rect_x in 0..32 {
                    let label = &(rect_x + 1).to_string();

                    Self::draw_route(ui, rect_x, 0, (0x20, 0xa4, 0xf7), label);
                    Self::draw_route(ui, rect_x, 1, (0x00, 0x73, 0xff), label);
                    Self::draw_route(ui, rect_x, 2, (0xff, 0xb8, 0x00), label);

                    Self::draw_route(ui, rect_x, 11, (0x20, 0xa4, 0xf7), label);
                    Self::draw_route(ui, rect_x, 12, (0x00, 0x73, 0xff), label);
                    Self::draw_route(ui, rect_x, 13, (0xff, 0xb8, 0x00), label);

                    Self::draw_route(ui, rect_x, 17, (0x7b, 0x7b, 0x7b), "M");

                    if rect_x >= 8 {
                        continue;
                    }

                    Self::draw_route(ui, rect_x, 4, (0xff, 0x00, 0x37), label);
                    Self::draw_route(ui, rect_x, 15, (0x7b, 0x7b, 0x7b), "M");

                    if rect_x >= 2 {
                        continue;
                    }

                    Self::draw_route(ui, rect_x, 5, (0xd9, 0x24, 0x94), label);
                    Self::draw_route(ui, rect_x, 6, (0x72, 0xab, 0x20), label);

                    Self::draw_route(ui, rect_x, 16, (0x7b, 0x7b, 0x7b), "M");
                }

                Self::draw_text(ui, 1134, 101, (0x95, 0x95, 0x95), "-43 dB");
            });
    }

    fn draw_route(ui: &mut egui::Ui, rect_x: i16, rect_y: i16, color: (u8, u8, u8), text: &str) {
        let pos_x = 128 + rect_x * 30;
        let pos_y = 285 + rect_y * 15;

        ui.painter().rect_filled(
            egui::Rect::from_min_size(
                egui::pos2(pos_x as f32, pos_y as f32),
                egui::vec2(29.0, 14.0),
            ),
            0.0,
            egui::Color32::from_rgb(color.0, color.1, color.2),
        );

        Self::draw_text(ui, pos_x + 15, pos_y, (0xff, 0xff, 0xff), text);
    }

    fn draw_text(ui: &mut egui::Ui, center_x: i16, pos_y: i16, color: (u8, u8, u8), text: &str) {
        let color = egui::Color32::from_rgb(color.0, color.1, color.2);
        let label = egui::Label::new(egui::RichText::new(text).strong().size(12.0).color(color));
        let (_, galley, _) = label.layout_in_ui(ui);

        let pos_x = center_x as f32 - galley.size().x / 2.0;

        ui.painter().galley(
            egui::pos2(pos_x, pos_y as f32),
            galley,
            egui::Color32::PLACEHOLDER,
        );
    }
}

impl eframe::App for Orion32ControlPanel {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.ui_content(ui);
    }
}

fn main() -> eframe::Result {
    let hid_api = HidApi::new().expect("Failed to create API instance");
    let orion_hid = hid_api.open(0x23e5, 0xa040).expect("Failed to open device");

    let parameters = Arc::new(Mutex::new(Orion32Parameters::default()));
    let parameters_reader = Arc::clone(&parameters);

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

            let ctx = cc.egui_ctx.clone();

            thread::spawn(move || {
                let mut hid_data = [0u8; 64];

                loop {
                    orion_hid
                        .read(&mut hid_data)
                        .expect("Failed to read HID packet");

                    match hid_data[31] {
                        1 => {
                            let mut parameters = parameters_reader.lock().unwrap();
                            parameters.vu_left.copy_from_slice(&hid_data[32..]);

                            ctx.request_repaint();
                        }
                        2 => {
                            let mut parameters = parameters_reader.lock().unwrap();
                            parameters.vu_right.copy_from_slice(&hid_data[32..]);

                            ctx.request_repaint();
                        }
                        _ => {}
                    }
                }
            });

            Ok(Box::new(Orion32ControlPanel { parameters }))
        }),
    )
}
