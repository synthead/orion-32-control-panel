// Hide console window on Windows.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;
use std::sync::Arc;
use std::sync::Mutex;

use crate::orion_hid::OrionHid;
use crate::orion_state::OrionState;

struct ControlPanelRoute {
    source: usize,
    label: String,
}

struct ControlPanelRoutes {
    adc: [ControlPanelRoute; 32],
    usb_play: [ControlPanelRoute; 32],
    madi_in: [ControlPanelRoute; 64],
    adat_in: [ControlPanelRoute; 16],
    spdif_in: [ControlPanelRoute; 2],
    mix_lr: [ControlPanelRoute; 2],

    dac: [ControlPanelRoute; 32],
    usb_rec: [ControlPanelRoute; 32],
    madi_out: [ControlPanelRoute; 64],
    adat_out: [ControlPanelRoute; 16],
    spdif_out: [ControlPanelRoute; 2],
    mix_ch: [ControlPanelRoute; 32],
}

pub struct ControlPanel {
    pub orion_state: Arc<Mutex<OrionState>>,
    pub orion_hid: OrionHid,
    routes: ControlPanelRoutes,
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

    const ROUTE_COLORS: [egui::Color32; 6] = [
        egui::Color32::from_rgb(0x20, 0xa4, 0xf7),
        egui::Color32::from_rgb(0x00, 0x73, 0xff),
        egui::Color32::from_rgb(0xff, 0xb8, 0x00),
        egui::Color32::from_rgb(0xff, 0x00, 0x37),
        egui::Color32::from_rgb(0xd9, 0x24, 0x94),
        egui::Color32::from_rgb(0x72, 0xab, 0x20),
    ];

    pub fn new(orion_state: Arc<Mutex<OrionState>>, orion_hid: OrionHid) -> Self {
        let routes = ControlPanelRoutes {
            adc: Self::route_labels_init(0),
            usb_play: Self::route_labels_init(1),
            madi_in: Self::route_labels_init(2),
            adat_in: Self::route_labels_init(3),
            spdif_in: Self::route_labels_init(4),
            mix_lr: Self::route_labels_init(5),

            dac: Self::route_labels_init(1),
            usb_rec: Self::route_labels_init(0),
            madi_out: Self::route_labels_init(2),
            adat_out: Self::route_labels_init(3),
            spdif_out: Self::route_labels_init(4),
            mix_ch: Self::route_labels_init(0),
        };

        Self {
            orion_state,
            orion_hid,
            routes,
            editing_id: None,
        }
    }

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

                let orion_state = self.orion_state.lock().unwrap().clone();

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

                let madi_count = match orion_state.sample_rate {
                    3 | 4 => 32,
                    5 | 6 => 16,
                    _ => 64,
                };

                let adat_count = match orion_state.sample_rate {
                    3 | 4 => 8,
                    5 | 6 => 4,
                    _ => 16,
                };

                self.draw_routes(ui, 0, 32, &self.routes.adc);
                self.draw_routes(ui, 1, 32, &self.routes.usb_play);
                self.draw_routes(ui, 2, madi_count, &self.routes.madi_in);
                self.draw_routes(ui, 4, adat_count, &self.routes.adat_in);
                self.draw_routes(ui, 5, 2, &self.routes.spdif_in);
                self.draw_routes(ui, 6, 2, &self.routes.mix_lr);

                self.draw_routes(ui, 11, 32, &self.routes.dac);
                self.draw_routes(ui, 12, 32, &self.routes.usb_rec);
                self.draw_routes(ui, 13, madi_count, &self.routes.madi_out);
                self.draw_routes(ui, 15, adat_count, &self.routes.adat_out);
                self.draw_routes(ui, 16, 2, &self.routes.spdif_out);
                self.draw_routes(ui, 17, 32, &self.routes.mix_ch);

                let line_out_value = (((orion_state.line_out) as f32 / 255.0) * -96.0) as i8;
                let line_out_text = format!("{} dB", line_out_value);

                Self::draw_text(ui, 1134, 101, (0x95, 0x95, 0x95), &line_out_text);

                let serial = String::from_utf8_lossy(&orion_state.serial);

                Self::draw_combobox(ui, 950, 7, 110, 0, &[&serial], |_selected| {
                    // TODO: Implement multi-interface control, and switch it here.
                });

                Self::draw_combobox(
                    ui,
                    396,
                    38,
                    83,
                    orion_state.left_vu_source,
                    &Self::VU_SELECTIONS,
                    |selected| {
                        self.orion_hid.queue_vu_source_left(selected);
                    },
                );

                Self::draw_combobox(
                    ui,
                    977,
                    38,
                    83,
                    orion_state.right_vu_source,
                    &Self::VU_SELECTIONS,
                    |selected| {
                        self.orion_hid.queue_vu_source_right(selected);
                    },
                );

                Self::draw_combobox(
                    ui,
                    508,
                    92,
                    72,
                    orion_state.clock_source,
                    &Self::CLOCK_SOURCE_SELECTIONS,
                    |selected| {
                        self.orion_hid.queue_clock_source(selected);
                    },
                );

                Self::draw_combobox(
                    ui,
                    597,
                    92,
                    72,
                    orion_state.sample_rate,
                    &Self::SAMPLE_RATE_SELECTIONS,
                    |selected| {
                        self.orion_hid.queue_sample_rate(selected);
                    },
                );
            });
    }

    fn draw_routes(
        &self,
        ui: &mut egui::Ui,
        start_rect_y: u8,
        rect_count: u8,
        routes: &[ControlPanelRoute],
    ) {
        for rect_i in 0..rect_count {
            let rect_x = rect_i % 32;
            let rect_y = start_rect_y + rect_i / 32;

            let pos_x = 128 + rect_x as u16 * 30;
            let pos_y = 285 + rect_y as u16 * 15;

            let route = &routes[rect_i as usize];

            ui.painter().rect_filled(
                egui::Rect::from_min_size(
                    egui::pos2(pos_x as f32, pos_y as f32),
                    egui::vec2(29.0, 14.0),
                ),
                0.0,
                Self::ROUTE_COLORS[route.source],
            );

            Self::draw_text(
                ui,
                pos_x + 15,
                pos_y,
                (0xff, 0xff, 0xff),
                route.label.as_str(),
            );
        }
    }

    fn draw_text(ui: &mut egui::Ui, center_x: u16, pos_y: u16, color: (u8, u8, u8), text: &str) {
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
        selected: u8,
        selections: &[&str],
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

                let mut selected_mut = selected;

                egui::ComboBox::from_id_salt((x, y))
                    .width(width as f32)
                    .selected_text(egui::RichText::new(selections[selected as usize]).size(10.5))
                    .show_ui(ui, |ui| {
                        for (i, selection) in selections.iter().enumerate() {
                            let response =
                                ui.selectable_value(&mut selected_mut, i as u8, *selection);

                            if response.changed() {
                                changed_callback(selected_mut);
                            }
                        }
                    });
            },
        );
    }

    fn route_labels_init<const N: usize>(source: usize) -> [ControlPanelRoute; N] {
        std::array::from_fn(|i| ControlPanelRoute {
            source,
            label: (i + 1).to_string(),
        })
    }
}

impl eframe::App for ControlPanel {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.ui_content(ui);
    }
}
