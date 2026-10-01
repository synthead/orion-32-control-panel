// Hide console window on Windows.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;
use hidapi::HidApi;
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Default)]
struct Orion32Parameters {
    serial: String,
    left_vu_source: u8,
    left_vu_values: [u8; 32],
    right_vu_source: u8,
    right_vu_values: [u8; 32],
    clock_source: u8,
    sample_rate: u8,
    line_out: u8,
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
    const VU_SELECTIONS: [&str; 14] = [
        "ADC",
        "USB PLAY",
        "MADI IN 1",
        "MADI IN 2",
        "ADAT IN",
        "S/PDIF IN",
        "MIX 1 L/R",
        "DAC",
        "USB REC",
        "MADI OUT 1",
        "MADI OUT 2",
        "ADAT OUT",
        "S/PDIF OUT",
        "MIX 1 CH",
    ];

    const CLOCK_SOURCE_SELECTIONS: [&str; 9] = [
        "Oven", "W.C.", "ADAT", "ADAT 2x", "ADAT 4x", "MADI", "MADI 2x", "MADI 4x", "S/PDIF",
    ];

    const SAMPLE_RATE_SELECTIONS: [&str; 7] = [
        "32.0 kHz",
        "44.1 kHz",
        "48.0 kHz",
        "88.2 kHz",
        "96.0 kHz",
        "176.4 kHz",
        "192.0 kHz",
    ];

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

                let mut parameters = self.parameters.lock().unwrap();

                for peak_x in 0..32 {
                    let pos_x_1 = 135 + peak_x * 11;
                    let pos_x_2 = pos_x_1 + 581;

                    render_peak!(
                        ui,
                        pos_x_1,
                        60,
                        parameters.left_vu_values[peak_x],
                        "MonitorPeakMeter/default/background"
                    );
                    render_peak!(
                        ui,
                        pos_x_2,
                        60,
                        parameters.right_vu_values[peak_x],
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

                let line_out_value = (((parameters.line_out) as f32 / 255.0) * -96.0) as i8;
                let line_out_text = format!("{} dB", line_out_value);
                Self::draw_text(ui, 1134, 101, (0x95, 0x95, 0x95), &line_out_text);

                Self::draw_combobox(
                    ui,
                    950,
                    7,
                    110,
                    &mut 0,
                    &[parameters.serial.as_str()],
                    "serial",
                );

                Self::draw_combobox(
                    ui,
                    396,
                    38,
                    83,
                    &mut parameters.left_vu_source,
                    &Self::VU_SELECTIONS,
                    "left_vu_source",
                );

                Self::draw_combobox(
                    ui,
                    977,
                    38,
                    83,
                    &mut parameters.right_vu_source,
                    &Self::VU_SELECTIONS,
                    "right_vu_source",
                );

                Self::draw_combobox(
                    ui,
                    508,
                    92,
                    72,
                    &mut parameters.clock_source,
                    &Self::CLOCK_SOURCE_SELECTIONS,
                    "clock_source",
                );

                Self::draw_combobox(
                    ui,
                    597,
                    92,
                    72,
                    &mut parameters.sample_rate,
                    &Self::SAMPLE_RATE_SELECTIONS,
                    "sample_rate",
                );
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

    fn draw_combobox(
        ui: &mut egui::Ui,
        x: u16,
        y: u16,
        width: u8,
        selected: &mut u8,
        selections: &[&str],
        id: &str,
    ) {
        ui.scope_builder(
            egui::UiBuilder::new().max_rect(egui::Rect::from_min_size(
                egui::pos2(x as f32, y as f32),
                egui::vec2(width as f32, 17.0),
            )),
            |ui| {
                ui.visuals_mut().widgets.inactive.weak_bg_fill = egui::Color32::BLACK;
                ui.visuals_mut().widgets.hovered.weak_bg_fill = egui::Color32::BLACK;
                ui.visuals_mut().widgets.active.weak_bg_fill = egui::Color32::BLACK;
                ui.visuals_mut().widgets.open.weak_bg_fill = egui::Color32::BLACK;

                egui::ComboBox::from_id_salt(id)
                    .width(width as f32)
                    .selected_text(egui::RichText::new(selections[*selected as usize]).size(10.5))
                    .show_ui(ui, |ui| {
                        for (i, selection) in selections.iter().enumerate() {
                            ui.selectable_value(selected, i as u8, *selection);
                        }
                    });
            },
        );
    }

    fn get_init_values(orion_hid: &hidapi::HidDevice) {
        let packet_init = [
            0x74, 0x00, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];

        orion_hid
            .write(&packet_init)
            .expect("Failed to write HID packet");
    }

    fn update_loop(
        orion_hid: hidapi::HidDevice,
        parameters: Arc<Mutex<Orion32Parameters>>,
        ctx: egui::Context,
    ) {
        let mut hid_data = [0u8; 64];

        loop {
            orion_hid
                .read(&mut hid_data)
                .expect("Failed to read HID packet");

            if matches!(hid_data[0], 0x73 | 0x75) {
                let mut parameters_locked = parameters.lock().unwrap();

                if hid_data[0] == 0x73 {
                    parameters_locked.left_vu_source = hid_data[30] & 0x0f;
                    parameters_locked.right_vu_source = hid_data[30] >> 4;
                    parameters_locked.clock_source = hid_data[19];
                    parameters_locked.sample_rate = hid_data[18];

                    parameters_locked.line_out = hid_data[28];

                    let vu = if hid_data[31] == 1 {
                        &mut parameters_locked.left_vu_values
                    } else {
                        &mut parameters_locked.right_vu_values
                    };

                    vu.copy_from_slice(&hid_data[32..]);
                }

                if hid_data[0] == 0x75 {
                    parameters_locked.serial =
                        std::str::from_utf8(&hid_data[36..49]).unwrap().to_owned();
                }

                ctx.request_repaint();
            }
        }
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

            let ctx_copy = cc.egui_ctx.clone();
            let parameters_copy = Arc::clone(&parameters);

            thread::spawn(move || {
                Orion32ControlPanel::get_init_values(&orion_hid);
                Orion32ControlPanel::update_loop(orion_hid, parameters_copy, ctx_copy);
            });

            Ok(Box::new(Orion32ControlPanel { parameters }))
        }),
    )
}
