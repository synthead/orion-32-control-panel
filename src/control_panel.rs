use std::sync::Arc;
use std::sync::Mutex;

use crate::orion_hid::OrionHid;
use crate::orion_state::OrionState;

use crate::drag_value::DragValue;
use crate::text_editor::TextEditor;

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
    const PRESET: egui::Color32 = egui::Color32::from_rgb(0x95, 0x95, 0x95);
    const PRESET_SEL: egui::Color32 = egui::Color32::from_rgb(0x64, 0xe7, 0x1d);
}

struct ControlPanelDimensions;

impl ControlPanelDimensions {
    const LINE_OUT_KNOB_X: u16 = 1099;
    const LINE_OUT_KNOB_Y: u16 = 74;
    const LINE_OUT_KNOB_RAD: f32 = 2.3;
    const LINE_OUT_KNOB_TRAVEL: u16 = 250;

    const LINE_OUT_KNOB_HITBOX_X: f32 = 1099.0;
    const LINE_OUT_KNOB_HITBOX_Y: f32 = 69.0;
    const LINE_OUT_KNOB_HITBOX_WIDTH: f32 = 69.0;
    const LINE_OUT_KNOB_HITBOX_HEIGHT: f32 = 69.0;

    const LINE_OUT_TEXT_HITBOX_X: f32 = 1118.0;
    const LINE_OUT_TEXT_HITBOX_Y: f32 = 101.0;
    const LINE_OUT_TEXT_HITBOX_WIDTH: f32 = 31.0;
    const LINE_OUT_TEXT_HITBOX_HEIGHT: f32 = 14.0;

    const PRESET_X: f32 = 820.0;
    const PRESET_Y: f32 = 209.0;
    const PRESET_STEP: f32 = 65.0;
    const PRESET_WIDTH: f32 = 50.0;
    const PRESET_HEIGHT: f32 = 12.0;

    const STANDBY_X: u16 = 20;
    const STANDBY_Y: u16 = 8;

    const FONT_SIZE_LINE_OUT: f32 = 11.0;
    const FONT_SIZE_PRESET: f32 = 9.5;
}

pub(crate) struct ControlPanel {
    orion_state: Arc<Mutex<OrionState>>,
    orion_hid: Arc<Mutex<OrionHid>>,

    routing: ControlPanelRouting,
    mixer: ControlPanelMixer,
    ad_phase: ControlPanelAdPhase,

    text_editor: Arc<Mutex<TextEditor>>,
    drag_value: Arc<Mutex<DragValue>>,

    current_tab: u8,
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

    // TODO: Support "Oven" turning into "10M" when 10M clock is connected.
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

    pub fn new(
        orion_state: Arc<Mutex<OrionState>>,
        orion_hid: Arc<Mutex<OrionHid>>,
        text_editor: Arc<Mutex<TextEditor>>,
        drag_value: Arc<Mutex<DragValue>>,
    ) -> Self {
        let routing = ControlPanelRouting::new(
            Arc::clone(&orion_state),
            Arc::clone(&orion_hid),
            Arc::clone(&text_editor),
        );

        let mixer = ControlPanelMixer::new(
            Arc::clone(&orion_state),
            Arc::clone(&orion_hid),
            Arc::clone(&text_editor),
            Arc::clone(&drag_value),
        );

        let ad_phase = ControlPanelAdPhase::new(Arc::clone(&orion_state), Arc::clone(&orion_hid));

        Self {
            orion_state,
            orion_hid,

            routing,
            mixer,
            ad_phase,

            text_editor,
            drag_value,

            current_tab: 0,
        }
    }

    fn ui_content(&mut self, ui: &mut egui::Ui) {
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ui, |ui| {
                let orion_state = self.orion_state.lock().unwrap().clone();

                Self::draw_base_graphics(ui);
                self.draw_standby(ui, orion_state.preset_standby);
                self.draw_tab_buttons(ui);
                self.draw_presets(ui, orion_state.preset_standby);

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
                    1 => self.mixer.draw(ui),
                    2 => self.ad_phase.draw(ui),
                    _ => self.routing.draw(ui),
                };
            });
    }

    fn draw_base_graphics(ui: &mut egui::Ui) {
        render_asset!(ui, 0, 0, "ControlPanel/default/background", 1.0, 0.0, None);

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

    fn draw_standby(&mut self, ui: &mut egui::Ui, preset_standby: u8) {
        let powered_on = preset_standby & 0x08 != 0;

        let standby_button = if powered_on {
            render_asset!(
                ui,
                ControlPanelDimensions::STANDBY_X,
                ControlPanelDimensions::STANDBY_Y,
                "StandbyButton/default/pressed",
                1.0,
                0.0,
                Some(egui::Sense::click())
            )
        } else {
            render_asset!(
                ui,
                ControlPanelDimensions::STANDBY_X,
                ControlPanelDimensions::STANDBY_Y,
                "StandbyButton/default/released",
                1.0,
                0.0,
                Some(egui::Sense::click())
            )
        };

        if standby_button.clicked() {
            let inverted_power = (!powered_on) as u8;

            self.orion_hid.lock().unwrap().queue_standby(inverted_power);
        }
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

    fn draw_presets(&mut self, ui: &mut egui::Ui, preset_standby: u8) {
        for preset in 1..6 {
            let label = format!("PRESET {}", preset);
            let rect = egui::Rect::from_min_size(
                egui::pos2(
                    ControlPanelDimensions::PRESET_X
                        + (preset - 1) as f32 * ControlPanelDimensions::PRESET_STEP,
                    ControlPanelDimensions::PRESET_Y,
                ),
                egui::vec2(
                    ControlPanelDimensions::PRESET_WIDTH,
                    ControlPanelDimensions::PRESET_HEIGHT,
                ),
            );

            let response = ui.interact(
                rect,
                ui.id().with(&format!("preset_{preset}")),
                egui::Sense::click(),
            );

            let color = if preset == preset_standby & 0x07 || response.hovered() {
                ControlPanelColors::PRESET_SEL
            } else {
                ControlPanelColors::PRESET
            };

            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                label,
                egui::FontId::proportional(ControlPanelDimensions::FONT_SIZE_PRESET),
                color,
            );

            if response.clicked() {
                let mut orion_hid = self.orion_hid.lock().unwrap();

                if ui.input(|i| i.modifiers.ctrl) {
                    orion_hid.queue_preset_write(preset);

                    return;
                }

                orion_hid.queue_preset(preset);

                orion_hid.queue_get_routes();
                orion_hid.queue_get_mixer_levels();
                orion_hid.queue_get_mixer_pans();
                orion_hid.queue_get_ad_phases();
            }
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
        let line_out_knob_hitbox_rect = egui::Rect::from_min_size(
            egui::pos2(
                ControlPanelDimensions::LINE_OUT_KNOB_HITBOX_X,
                ControlPanelDimensions::LINE_OUT_KNOB_HITBOX_Y,
            ),
            egui::vec2(
                ControlPanelDimensions::LINE_OUT_KNOB_HITBOX_WIDTH,
                ControlPanelDimensions::LINE_OUT_KNOB_HITBOX_HEIGHT,
            ),
        );

        let (line_out, should_write) = self.drag_value.lock().unwrap().value_with_drag(
            ui,
            line_out_knob_hitbox_rect,
            line_out,
            0x00,
            0xff,
            ControlPanelDimensions::LINE_OUT_KNOB_TRAVEL,
            -1.0,
        );

        if should_write {
            self.orion_hid.lock().unwrap().queue_line_out(line_out);
        }

        let line_out_knob_rad =
            (0x80 as f32 - line_out as f32) / 127.0 * ControlPanelDimensions::LINE_OUT_KNOB_RAD;

        render_asset!(
            ui,
            ControlPanelDimensions::LINE_OUT_KNOB_X,
            ControlPanelDimensions::LINE_OUT_KNOB_Y,
            "LineOutVolumePot/default/layers/pointer",
            1.0,
            line_out_knob_rad,
            None
        );

        let line_out_text_hitbox_rect = egui::Rect::from_min_size(
            egui::pos2(
                ControlPanelDimensions::LINE_OUT_TEXT_HITBOX_X,
                ControlPanelDimensions::LINE_OUT_TEXT_HITBOX_Y,
            ),
            egui::vec2(
                ControlPanelDimensions::LINE_OUT_TEXT_HITBOX_WIDTH,
                ControlPanelDimensions::LINE_OUT_TEXT_HITBOX_HEIGHT,
            ),
        );

        let line_out_value = -(line_out as f32 * 96.0 / 255.0).round() as i16;
        let line_out_text = format!("{} dB", line_out_value);

        if let Some((text, tab)) = self.text_editor.lock().unwrap().draw(
            ui,
            line_out_text_hitbox_rect,
            "line_out",
            &line_out_text,
            ControlPanelDimensions::FONT_SIZE_LINE_OUT,
            ControlPanelColors::LINE_OUT,
        ) {
            if tab.is_none() {
                if let Some(line_out_new) = Self::line_out_for_text(&text) {
                    self.orion_hid.lock().unwrap().queue_line_out(line_out_new);
                }
            }
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
