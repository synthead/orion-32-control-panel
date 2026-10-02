// Hide console window on Windows.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;
use std::sync::Arc;
use std::sync::Mutex;

use crate::orion_hid::OrionHid;
use crate::orion_state::OrionState;

pub struct ControlPanel {
    pub orion_state: Arc<Mutex<OrionState>>,
    pub orion_hid: OrionHid,
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

impl ControlPanel {
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

                let mut orion_state = self.orion_state.lock().unwrap();

                for peak_x in 0..32 {
                    let pos_x_1 = 135 + peak_x * 11;
                    let pos_x_2 = pos_x_1 + 581;

                    render_peak!(
                        ui,
                        pos_x_1,
                        60,
                        orion_state.left_vu_values[peak_x],
                        "MonitorPeakMeter/default/background"
                    );
                    render_peak!(
                        ui,
                        pos_x_2,
                        60,
                        orion_state.right_vu_values[peak_x],
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

                let line_out_value = (((orion_state.line_out) as f32 / 255.0) * -96.0) as i8;
                let line_out_text = format!("{} dB", line_out_value);

                Self::draw_text(ui, 1134, 101, (0x95, 0x95, 0x95), &line_out_text);

                let serial = String::from_utf8_lossy(&orion_state.serial);

                Self::draw_combobox(ui, 950, 7, 110, &mut 0, &[&serial], "serial", |selected| {});

                Self::draw_combobox(
                    ui,
                    396,
                    38,
                    83,
                    &mut orion_state.left_vu_source,
                    &Self::VU_SELECTIONS,
                    "left_vu_source",
                    |selected| {},
                );

                Self::draw_combobox(
                    ui,
                    977,
                    38,
                    83,
                    &mut orion_state.right_vu_source,
                    &Self::VU_SELECTIONS,
                    "right_vu_source",
                    |selected| {},
                );

                Self::draw_combobox(
                    ui,
                    508,
                    92,
                    72,
                    &mut orion_state.clock_source,
                    &Self::CLOCK_SOURCE_SELECTIONS,
                    "clock_source",
                    |selected| {},
                );

                Self::draw_combobox(
                    ui,
                    597,
                    92,
                    72,
                    &mut orion_state.sample_rate,
                    &Self::SAMPLE_RATE_SELECTIONS,
                    "sample_rate",
                    |selected| {
                        self.orion_hid.queue_sample_rate(selected);
                    },
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
        mut changed_callback: impl FnMut(u8),
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
                            let response = ui.selectable_value(selected, i as u8, *selection);

                            if response.changed() {
                                changed_callback(*selected);
                            }
                        }
                    });
            },
        );
    }
}

impl eframe::App for ControlPanel {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.ui_content(ui);
    }
}
