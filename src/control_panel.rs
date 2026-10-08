use std::sync::Arc;
use std::sync::Mutex;

use crate::orion_hid::OrionHid;
use crate::orion_state::OrionState;

use crate::control_panel_ad_phase::ControlPanelAdPhase;
use crate::control_panel_helpers::ControlPanelHelpers;
use crate::control_panel_mixer::ControlPanelMixer;
use crate::control_panel_routing::ControlPanelRouting;

use crate::render_asset;

macro_rules! render_vu_meter {
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

struct ControlPanelColors;

impl ControlPanelColors {
    const LINE_OUT: egui::Color32 = egui::Color32::from_rgb(0x95, 0x95, 0x95);
}

struct ControlPanelDimensions;

impl ControlPanelDimensions {
    const LINE_OUT_HITBOX_X: f32 = 1118.0;
    const LINE_OUT_HITBOX_Y: f32 = 101.0;
    const LINE_OUT_HITBOX_WIDTH: f32 = 31.0;
    const LINE_OUT_HITBOX_HEIGHT: f32 = 14.0;

    const FONT_SIZE_LINE_OUT: f32 = 11.0;
}

pub(crate) struct ControlPanel {
    orion_state: Arc<Mutex<OrionState>>,
    orion_hid: Arc<Mutex<OrionHid>>,

    routing: ControlPanelRouting,
    mixer: ControlPanelMixer,
    ad_phase: ControlPanelAdPhase,

    current_tab: u8,

    edit_line_out: bool,
    edit_string: String,
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
        let mixer = ControlPanelMixer::new(Arc::clone(&orion_state), Arc::clone(&orion_hid));
        let ad_phase = ControlPanelAdPhase::new(Arc::clone(&orion_state), Arc::clone(&orion_hid));

        Self {
            orion_state,
            orion_hid,

            routing,
            mixer,
            ad_phase,

            current_tab: 0,

            edit_line_out: false,
            edit_string: String::new(),
        }
    }

    fn ui_content(&mut self, ui: &mut egui::Ui) {
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ui, |ui| {
                let orion_state = self.orion_state.lock().unwrap().clone();

                Self::draw_base_graphics(ui);
                self.draw_tab_buttons(ui);
                Self::draw_preset_labels(ui);

                Self::draw_vu_meters(
                    ui,
                    &orion_state.left_vu_values,
                    &orion_state.right_vu_values,
                );

                self.draw_clock_comboboxes(ui, orion_state.clock_source, orion_state.sample_rate);

                self.draw_vu_meter_comboboxes(
                    ui,
                    orion_state.left_vu_source,
                    orion_state.right_vu_source,
                );

                Self::draw_serial_combobox(ui, &orion_state.serial);
                self.draw_line_out(ui, orion_state.line_out);

                match self.current_tab {
                    1 => {
                        self.mixer.draw(ui);
                    }
                    2 => {
                        self.ad_phase.draw(ui);
                    }
                    _ => {
                        self.routing.draw(ui);
                    }
                };
            });
    }

    fn draw_base_graphics(ui: &mut egui::Ui) {
        render_asset!(ui, 0, 0, "ControlPanel/default/background", 1.0, 0.0, None);

        render_asset!(ui, 20, 8, "StandbyButton/default/pressed", 1.0, 0.0, None);
        render_asset!(
            ui,
            1075,
            9,
            "SettingsButton/default/released",
            1.0,
            0.0,
            None
        );
        render_asset!(
            ui,
            1124,
            9,
            "MinimizeButton/default/released",
            1.0,
            0.0,
            None
        );
        render_asset!(ui, 1100, 9, "AboutButton/default/released", 1.0, 0.0, None);
        render_asset!(ui, 1149, 9, "CloseButton/default/released", 1.0, 0.0, None);

        render_asset!(
            ui,
            79,
            683,
            "FeedLeftArrow/default/released",
            1.0,
            0.0,
            None
        );
        render_asset!(
            ui,
            1093,
            683,
            "FeedRightArrow/default/released",
            1.0,
            0.0,
            None
        );
    }

    fn draw_tab_buttons(&mut self, ui: &mut egui::Ui) {
        let routing_button = if self.current_tab == 0 {
            render_asset!(
                ui,
                82,
                209,
                "RoutingNavButton/default/pressed@2x",
                0.5,
                0.0,
                Some(egui::Sense::click())
            )
        } else {
            render_asset!(
                ui,
                82,
                209,
                "RoutingNavButton/default/released@2x",
                0.5,
                0.0,
                Some(egui::Sense::click())
            )
        };

        let mixer_button = if self.current_tab == 1 {
            render_asset!(
                ui,
                178,
                209,
                "MixerNavButton/default/pressed@2x",
                0.5,
                0.0,
                Some(egui::Sense::click())
            )
        } else {
            render_asset!(
                ui,
                178,
                209,
                "MixerNavButton/default/released@2x",
                0.5,
                0.0,
                Some(egui::Sense::click())
            )
        };

        let ad_phase_button = if self.current_tab == 2 {
            render_asset!(
                ui,
                257,
                209,
                "PhaseInvNavButton/default/pressed",
                1.0,
                0.0,
                Some(egui::Sense::click())
            )
        } else {
            render_asset!(
                ui,
                257,
                209,
                "PhaseInvNavButton/default/released",
                1.0,
                0.0,
                Some(egui::Sense::click())
            )
        };

        if routing_button.clicked() {
            self.current_tab = 0;
        }

        if mixer_button.clicked() {
            self.current_tab = 1;
        }

        if ad_phase_button.clicked() {
            self.current_tab = 2;
        }
    }

    fn draw_preset_labels(ui: &mut egui::Ui) {
        for preset in 0..5 {
            let label = format!("PRESET {}", preset + 1);
            let pos_x = 845 + preset * 65;

            ControlPanelHelpers::draw_text(ui, pos_x, 210, (0x95, 0x95, 0x95), &label, 9.5);
        }
    }

    fn draw_vu_meters(ui: &mut egui::Ui, left_vu_values: &[u8; 32], right_vu_values: &[u8; 32]) {
        for vu_meter_x in 0..32 {
            let pos_x_1 = 135 + vu_meter_x * 11;
            let pos_x_2 = pos_x_1 + 581;

            render_vu_meter!(
                ui,
                pos_x_1,
                60,
                left_vu_values[vu_meter_x],
                "MonitorPeakMeter/default/background"
            );

            render_vu_meter!(
                ui,
                pos_x_2,
                60,
                right_vu_values[vu_meter_x],
                "MonitorPeakMeter/default/background"
            );
        }
    }

    fn draw_vu_meter_comboboxes(&self, ui: &mut egui::Ui, left_vu_source: u8, right_vu_source: u8) {
        ControlPanelHelpers::draw_combobox(
            ui,
            396,
            38,
            83,
            left_vu_source,
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
            right_vu_source,
            &Self::VU_SELECTIONS,
            |selected| {
                self.orion_hid
                    .lock()
                    .unwrap()
                    .queue_vu_source_right(selected);
            },
        );
    }

    fn draw_clock_comboboxes(&self, ui: &mut egui::Ui, clock_source: u8, sample_rate: u8) {
        ControlPanelHelpers::draw_combobox(
            ui,
            508,
            92,
            72,
            clock_source,
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
            sample_rate,
            &Self::SAMPLE_RATE_SELECTIONS,
            |selected| {
                self.orion_hid.lock().unwrap().queue_sample_rate(selected);
            },
        );
    }

    fn draw_line_out(&mut self, ui: &mut egui::Ui, line_out: u8) {
        let line_out_value = -(line_out as f32 * 96.0 / 255.0).round() as i16;
        let line_out_text = format!("{} dB", line_out_value);

        render_asset!(
            ui,
            1099,
            75,
            "LineOutVolumePot/default/layers/pointer",
            1.0,
            0.0,
            None
        );

        let hitbox_rect = egui::Rect::from_min_size(
            egui::pos2(
                ControlPanelDimensions::LINE_OUT_HITBOX_X,
                ControlPanelDimensions::LINE_OUT_HITBOX_Y,
            ),
            egui::vec2(
                ControlPanelDimensions::LINE_OUT_HITBOX_WIDTH,
                ControlPanelDimensions::LINE_OUT_HITBOX_HEIGHT,
            ),
        );

        let hitbox_id = ui.id().with("line_out_edit");
        let text_response = ui.interact(hitbox_rect, hitbox_id, egui::Sense::click());

        if text_response.double_clicked() {
            self.edit_line_out = true;
            self.edit_string = line_out_text.clone();
        }

        if !self.edit_line_out {
            ui.put(
                hitbox_rect,
                egui::Label::new(
                    egui::RichText::new(line_out_text)
                        .font(egui::FontId::proportional(
                            ControlPanelDimensions::FONT_SIZE_LINE_OUT,
                        ))
                        .color(ControlPanelColors::LINE_OUT),
                )
                .selectable(false),
            );

            return;
        }

        if ui.input(|i| {
            i.pointer.any_click()
                && i.pointer
                    .interact_pos()
                    .is_some_and(|pos| !hitbox_rect.contains(pos))
        }) {
            if let Some(line_out_new) = Self::line_out_for_text(&self.edit_string) {
                self.orion_hid.lock().unwrap().queue_line_out(line_out_new);
            }

            self.edit_line_out = false;
        }

        let response = ui.put(
            hitbox_rect,
            egui::TextEdit::singleline(&mut self.edit_string)
                .id(hitbox_id)
                .font(egui::FontId::proportional(
                    ControlPanelDimensions::FONT_SIZE_LINE_OUT,
                ))
                .horizontal_align(egui::Align::Center)
                .vertical_align(egui::Align::Center)
                .text_color(ControlPanelColors::LINE_OUT)
                .frame(egui::Frame::NONE),
        );

        if response.has_focus() && response.gained_focus() {
            let mut state =
                egui::widgets::text_edit::TextEditState::load(ui.ctx(), hitbox_id).unwrap();

            let label_chars = self.edit_string.chars().count();

            state
                .cursor
                .set_char_range(Some(egui::text::CCursorRange::two(
                    egui::text::CCursor::new(0),
                    egui::text::CCursor::new(label_chars),
                )));

            state.store(ui.ctx(), hitbox_id);
        }

        if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            if let Some(line_out_new) = Self::line_out_for_text(&self.edit_string) {
                self.orion_hid.lock().unwrap().queue_line_out(line_out_new);
            }

            self.edit_line_out = false;
        }

        if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            println!("hit esc");
            self.edit_line_out = false;
        }
    }

    fn draw_serial_combobox(ui: &mut egui::Ui, serial: &[u8; 13]) {
        let serial_text = String::from_utf8_lossy(serial);

        ControlPanelHelpers::draw_combobox(ui, 950, 7, 110, 0, &[&serial_text], |_selected| {
            // TODO: Implement multi-interface control, and switch it here.
        });
    }

    fn line_out_for_text(text: &String) -> Option<u8> {
        let text_parsed = text.trim();
        let text_parsed = text_parsed.strip_suffix("dB").unwrap_or(text_parsed);
        let text_parsed = text_parsed.strip_suffix("db").unwrap_or(text_parsed);
        let text_parsed = text_parsed.trim();

        if text_parsed == "-inf" {
            return Some(0xff);
        }

        let text_parsed = match text_parsed.parse::<i16>() {
            Ok(value) => value,
            Err(_) => return None,
        };

        if text_parsed < -96 || text_parsed > 0 {
            return None;
        }

        Some((text_parsed.abs() * 0xff / 96) as u8)
    }
}

impl eframe::App for ControlPanel {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.ui_content(ui);
    }
}
