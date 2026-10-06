use std::sync::Arc;
use std::sync::Mutex;

use crate::orion_hid::OrionHid;
use crate::orion_state::OrionState;

use crate::control_panel_helpers::ControlPanelHelpers;
use crate::control_panel_routing::ControlPanelRouting;

use crate::render_asset;

macro_rules! render_peak {
    ($ui:expr, $x:expr, $y:expr, $db:expr, $subpath:literal) => {{
        let image = egui::Image::new(egui::include_image!(concat!(
            "../assets/images/",
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

pub(crate) struct ControlPanel {
    orion_state: Arc<Mutex<OrionState>>,
    orion_hid: Arc<Mutex<OrionHid>>,

    routing: ControlPanelRouting,
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

    pub fn new(orion_state: Arc<Mutex<OrionState>>, orion_hid: Arc<Mutex<OrionHid>>) -> Self {
        let routing = ControlPanelRouting::new(Arc::clone(&orion_state), Arc::clone(&orion_hid));

        Self {
            orion_state,
            orion_hid,

            routing,
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

                let line_out_value = (((orion_state.line_out) as f32 / 255.0) * -96.0) as i8;
                let line_out_text = format!("{} dB", line_out_value);

                Self::draw_text(ui, 1134, 101, (0x95, 0x95, 0x95), &line_out_text);

                let serial = String::from_utf8_lossy(&orion_state.serial);

                ControlPanelHelpers::draw_combobox(ui, 950, 7, 110, 0, &[&serial], |_selected| {
                    // TODO: Implement multi-interface control, and switch it here.
                });

                ControlPanelHelpers::draw_combobox(
                    ui,
                    396,
                    38,
                    83,
                    orion_state.left_vu_source,
                    &Self::VU_SELECTIONS,
                    |selected| {
                        self.orion_hid
                            .lock()
                            .unwrap()
                            .queue_vu_source_left(selected);
                    },
                );

                ControlPanelHelpers::draw_combobox(
                    ui,
                    977,
                    38,
                    83,
                    orion_state.right_vu_source,
                    &Self::VU_SELECTIONS,
                    |selected| {
                        self.orion_hid
                            .lock()
                            .unwrap()
                            .queue_vu_source_right(selected);
                    },
                );

                ControlPanelHelpers::draw_combobox(
                    ui,
                    508,
                    92,
                    72,
                    orion_state.clock_source,
                    &Self::CLOCK_SOURCE_SELECTIONS,
                    |selected| {
                        self.orion_hid.lock().unwrap().queue_clock_source(selected);
                    },
                );

                ControlPanelHelpers::draw_combobox(
                    ui,
                    597,
                    92,
                    72,
                    orion_state.sample_rate,
                    &Self::SAMPLE_RATE_SELECTIONS,
                    |selected| {
                        self.orion_hid.lock().unwrap().queue_sample_rate(selected);
                    },
                );

                Self::draw_peaks(
                    ui,
                    &orion_state.left_vu_values,
                    &orion_state.right_vu_values,
                );

                self.routing.draw(ui);
            });
    }

    fn draw_peaks(ui: &mut egui::Ui, left_vu_values: &[u8; 32], right_vu_values: &[u8; 32]) {
        for peak_x in 0..32 {
            let pos_x_1 = 135 + peak_x * 11;
            let pos_x_2 = pos_x_1 + 581;

            render_peak!(
                ui,
                pos_x_1,
                60,
                left_vu_values[peak_x],
                "MonitorPeakMeter/default/background"
            );

            render_peak!(
                ui,
                pos_x_2,
                60,
                right_vu_values[peak_x],
                "MonitorPeakMeter/default/background"
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
}

impl eframe::App for ControlPanel {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.ui_content(ui);
    }
}
