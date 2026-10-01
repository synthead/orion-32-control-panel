// Hide console window on Windows.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;
use hidapi::HidApi;
use std::sync::{Arc, Mutex};
use std::thread;

struct Orion32ControlPanel {
    packet: Arc<Mutex<[u8; 64]>>,
}

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

                let packet = self.packet.lock().unwrap();
                let packet_text = format!("{:02X?}", &*packet);

                Self::draw_text(ui, 590, 600, (0xff, 0xff, 0xff), &packet_text);
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

    let packet = Arc::new(Mutex::new([0u8; 64]));
    let packet_reader = Arc::clone(&packet);

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
                let mut buffer = [0u8; 64];

                loop {
                    orion_hid
                        .read(&mut buffer)
                        .expect("Failed to read HID packet");

                    {
                        let mut packet = packet_reader.lock().unwrap();
                        packet.copy_from_slice(&buffer);
                    }

                    ctx.request_repaint();
                }
            });

            Ok(Box::new(Orion32ControlPanel { packet }))
        }),
    )
}
