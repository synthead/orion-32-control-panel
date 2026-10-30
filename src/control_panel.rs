use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

use crate::orion_hid::OrionHid;
use crate::orion_hid_poller::OrionHidPoller;
use crate::orion_state::OrionState;
use crate::orion_state::OrionStateInitSteps;

use crate::drag_value::DragValue;
use crate::ignore_value::IgnoreValue;
use crate::text_editor::TextEditor;

use crate::control_panel_ad_phase::ControlPanelAdPhase;
use crate::control_panel_devices::ControlPanelDevices;
use crate::control_panel_helpers::ControlPanelHelpers;
use crate::control_panel_mixer::ControlPanelMixer;
use crate::control_panel_routing::ControlPanelRouting;

use crate::control_panel_about::ControlPanelAbout;
use crate::control_panel_settings::ControlPanelSettings;

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
    const VIEWPORT_WIDTH: f32 = 1180.0;
    const VIEWPORT_HEIGHT: f32 = 700.0;
    const OVERLAY_TRANSPARENCY: u8 = 128;

    const STANDBY_X: u16 = 20;
    const STANDBY_Y: u16 = 8;

    const SERIAL_X: u16 = 950;
    const SERIAL_Y: u16 = 7;
    const SERIAL_WIDTH: u8 = 110;

    const WINDOW_BUTTON_SETTINGS_X: u16 = 1075;
    const WINDOW_BUTTON_ABOUT_X: u16 = 1100;
    const WINDOW_BUTTON_MINIMIZE_X: u16 = 1124;
    const WINDOW_BUTTON_CLOSE_X: u16 = 1149;
    const WINDOW_BUTTON_Y: u16 = 9;

    const VU_SOURCE_LEFT_X: u16 = 396;
    const VU_SOURCE_RIGHT_X: u16 = 977;
    const VU_SOURCE_Y: u16 = 38;
    const VU_SOURCE_WIDTH: u8 = 83;

    const VU_METER_LEFT_X: usize = 135;
    const VU_METER_RIGHT_X: usize = 716;
    const VU_METER_X_STEP: usize = 11;
    const VU_METER_Y: usize = 60;

    const CLOCK_CLOCK_SOURCE_X: u16 = 508;
    const CLOCK_SAMPLE_RATE_X: u16 = 597;
    const CLOCK_Y: u16 = 92;
    const CLOCK_WIDTH: u8 = 72;

    const TAB_BUTTON_ROUTING_X: u16 = 82;
    const TAB_BUTTON_MIXER_X: u16 = 178;
    const TAB_BUTTON_AD_PHASE_X: u16 = 257;
    const TAB_BUTTON_Y: u16 = 209;

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

    const FONT_SIZE_LINE_OUT: f32 = 11.0;
    const FONT_SIZE_PRESET: f32 = 9.5;
}

pub(crate) struct ControlPanel {
    orion_state: Arc<Mutex<OrionState>>,
    orion_hid: Arc<Mutex<OrionHid>>,
    orion_hid_poller: Arc<Mutex<OrionHidPoller>>,

    devices: ControlPanelDevices,
    routing: ControlPanelRouting,
    mixer: ControlPanelMixer,
    ad_phase: ControlPanelAdPhase,

    settings: ControlPanelSettings,
    about: ControlPanelAbout,

    text_editor: Arc<Mutex<TextEditor>>,
    drag_value: Arc<Mutex<DragValue>>,

    current_tab: u8,
    line_out_ignore: IgnoreValue,
}

impl ControlPanel {
    const WINDOW_TITLE: &str = "Orion 32 Control Panel";

    const VU_SOURCE_SELECTIONS: [&str; 14] = [
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

    pub fn new() -> Self {
        let orion_state = Arc::new(Mutex::new(OrionState::default()));
        let (hid_write_tx, hid_write_rx) = mpsc::channel::<[u8; 64]>();

        let orion_hid_poller = Arc::new(Mutex::new(OrionHidPoller::new(
            Arc::clone(&orion_state),
            hid_write_rx,
        )));

        let orion_hid = OrionHid::new(hid_write_tx, Arc::clone(&orion_state));

        let orion_hid = Arc::new(Mutex::new(orion_hid));
        let text_editor = Arc::new(Mutex::new(TextEditor::new()));
        let drag_value = Arc::new(Mutex::new(DragValue::new()));

        let devices = ControlPanelDevices::new(Arc::clone(&orion_state));

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

        let settings = ControlPanelSettings::new(
            Arc::clone(&orion_state),
            Arc::clone(&orion_hid),
            Arc::clone(&text_editor),
            Arc::clone(&drag_value),
        );

        let about = ControlPanelAbout::new(Arc::clone(&orion_state));

        Self {
            orion_state,
            orion_hid,
            orion_hid_poller,

            devices,
            routing,
            mixer,
            ad_phase,

            settings,
            about,

            text_editor,
            drag_value,

            current_tab: 0,
            line_out_ignore: IgnoreValue::new(),
        }
    }

    pub(crate) fn run() -> eframe::Result {
        let icon_image = image::load_from_memory(include_bytes!("../assets/icons/icon.png"))
            .unwrap()
            .into_rgba8();

        let (width, height) = icon_image.dimensions();

        let icon = egui::IconData {
            rgba: icon_image.into_raw(),
            width,
            height,
        };

        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([
                    ControlPanelDimensions::VIEWPORT_WIDTH,
                    ControlPanelDimensions::VIEWPORT_HEIGHT,
                ])
                .with_decorations(false)
                .with_icon(std::sync::Arc::new(icon)),
            ..Default::default()
        };

        let control_panel = Self::new();

        control_panel.init_config();

        eframe::run_native(
            Self::WINDOW_TITLE,
            options,
            Box::new(move |cc| {
                cc.egui_ctx.set_theme(egui::Theme::Dark);

                cc.egui_ctx.style_mut_of(cc.egui_ctx.theme(), |style| {
                    style.interaction.interact_radius = 0.0;
                });

                egui_extras::install_image_loaders(&cc.egui_ctx);

                let orion_hid_poller = Arc::clone(&control_panel.orion_hid_poller);
                let ctx = cc.egui_ctx.clone();

                thread::spawn(move || {
                    let mut orion_hid_poller = orion_hid_poller.lock().unwrap();

                    loop {
                        orion_hid_poller.poll_and_update_state();
                        ctx.request_repaint();
                    }
                });

                Ok(Box::new(control_panel))
            }),
        )
    }

    pub(crate) fn init_config(&self) {
        self.routing.create_config_dir();
        self.routing.read_persistence();
    }

    fn ui_content(&mut self, ui: &mut egui::Ui) {
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ui, |ui| {
                let orion_state = self.orion_state.lock().unwrap().clone();

                if orion_state.inits_completed
                    != OrionStateInitSteps::SERIAL_NUMBER
                        | OrionStateInitSteps::FIRMWARE_VERSION
                        | OrionStateInitSteps::ROUTES
                        | OrionStateInitSteps::MIXER_LEVELS
                        | OrionStateInitSteps::MIXER_PANS_SOLOS_MUTES
                        | OrionStateInitSteps::AD_PHASES
                {
                    self.devices.draw(ui);

                    return;
                }

                Self::draw_base_graphics(ui);

                self.draw_standby(ui, orion_state.preset_standby);
                self.draw_window_buttons(ui);

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

                if self.settings.enabled || self.about.enabled {
                    let overlay_rect = egui::Rect::from_min_size(
                        egui::pos2(0.0, 0.0),
                        egui::vec2(
                            ControlPanelDimensions::VIEWPORT_WIDTH,
                            ControlPanelDimensions::VIEWPORT_HEIGHT,
                        ),
                    );

                    ui.painter().rect_filled(
                        overlay_rect,
                        0.0,
                        egui::Color32::from_black_alpha(
                            ControlPanelDimensions::OVERLAY_TRANSPARENCY,
                        ),
                    );

                    let response = ui.allocate_rect(overlay_rect, egui::Sense::click());

                    if response.is_pointer_button_down_on()
                        && ui.input(|i| i.pointer.button_pressed(egui::PointerButton::Primary))
                    {
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
                    }

                    if self.settings.enabled {
                        self.settings.draw(ui);
                    } else if self.about.enabled {
                        self.about.draw(ui);
                    }
                }
            });
    }

    fn draw_base_graphics(ui: &mut egui::Ui) {
        let response = render_asset!(
            ui,
            0,
            0,
            "ControlPanel/default/background",
            1.0,
            0.0,
            Some(egui::Sense::click())
        );

        if !egui::Popup::is_any_open(ui.ctx())
            && response.is_pointer_button_down_on()
            && ui.input(|i| i.pointer.button_pressed(egui::PointerButton::Primary))
        {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
        }

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

    fn draw_window_buttons(&mut self, ui: &mut egui::Ui) {
        let settings_response = render_asset!(
            ui,
            ControlPanelDimensions::WINDOW_BUTTON_SETTINGS_X,
            ControlPanelDimensions::WINDOW_BUTTON_Y,
            "SettingsButton/default/released",
            1.0,
            0.0,
            Some(egui::Sense::click())
        );

        if self.settings.enabled
            || settings_response.is_pointer_button_down_on() != settings_response.hovered()
        {
            render_asset!(
                ui,
                ControlPanelDimensions::WINDOW_BUTTON_SETTINGS_X,
                ControlPanelDimensions::WINDOW_BUTTON_Y,
                "SettingsButton/default/pressed",
                1.0,
                0.0,
                None
            );
        }

        let about_response = render_asset!(
            ui,
            ControlPanelDimensions::WINDOW_BUTTON_ABOUT_X,
            ControlPanelDimensions::WINDOW_BUTTON_Y,
            "AboutButton/default/released",
            1.0,
            0.0,
            Some(egui::Sense::click())
        );

        if self.about.enabled
            || about_response.is_pointer_button_down_on() != about_response.hovered()
        {
            render_asset!(
                ui,
                ControlPanelDimensions::WINDOW_BUTTON_ABOUT_X,
                ControlPanelDimensions::WINDOW_BUTTON_Y,
                "AboutButton/default/pressed",
                1.0,
                0.0,
                None
            );
        }

        let minimize_response = render_asset!(
            ui,
            ControlPanelDimensions::WINDOW_BUTTON_MINIMIZE_X,
            ControlPanelDimensions::WINDOW_BUTTON_Y,
            "MinimizeButton/default/released",
            1.0,
            0.0,
            Some(egui::Sense::click())
        );

        if minimize_response.is_pointer_button_down_on() != minimize_response.hovered() {
            render_asset!(
                ui,
                ControlPanelDimensions::WINDOW_BUTTON_MINIMIZE_X,
                ControlPanelDimensions::WINDOW_BUTTON_Y,
                "MinimizeButton/default/pressed",
                1.0,
                0.0,
                None
            );
        }

        let close_response = render_asset!(
            ui,
            ControlPanelDimensions::WINDOW_BUTTON_CLOSE_X,
            ControlPanelDimensions::WINDOW_BUTTON_Y,
            "CloseButton/default/released",
            1.0,
            0.0,
            Some(egui::Sense::click())
        );

        if close_response.is_pointer_button_down_on() != close_response.hovered() {
            render_asset!(
                ui,
                ControlPanelDimensions::WINDOW_BUTTON_CLOSE_X,
                ControlPanelDimensions::WINDOW_BUTTON_Y,
                "CloseButton/default/pressed",
                1.0,
                0.0,
                None
            );
        }

        if settings_response.clicked() {
            self.settings.enabled = true;
        }

        if about_response.clicked() {
            self.about.enabled = true;
        }

        if minimize_response.clicked() {
            ui.ctx()
                .send_viewport_cmd(egui::ViewportCommand::Minimized(true));
        }

        if close_response.clicked() {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }

    fn draw_standby(&mut self, ui: &mut egui::Ui, preset_standby: u8) {
        let standby_enabled = preset_standby & 0x08 != 0;

        let response = render_asset!(
            ui,
            ControlPanelDimensions::STANDBY_X,
            ControlPanelDimensions::STANDBY_Y,
            "StandbyButton/default/released",
            1.0,
            0.0,
            Some(egui::Sense::click())
        );

        if (standby_enabled || response.is_pointer_button_down_on()) != response.hovered() {
            render_asset!(
                ui,
                ControlPanelDimensions::STANDBY_X,
                ControlPanelDimensions::STANDBY_Y,
                "StandbyButton/default/pressed",
                1.0,
                0.0,
                None
            );
        }

        if response.clicked() {
            let inverted_standby = (!standby_enabled) as u8;

            self.orion_hid
                .lock()
                .unwrap()
                .queue_standby(inverted_standby);
        }
    }

    fn draw_tab_buttons(&mut self, ui: &mut egui::Ui) {
        let routing_response = if self.current_tab == 0 {
            render_asset!(
                ui,
                ControlPanelDimensions::TAB_BUTTON_ROUTING_X,
                ControlPanelDimensions::TAB_BUTTON_Y,
                "RoutingNavButton/default/pressed@2x",
                0.5,
                0.0,
                Some(egui::Sense::click())
            )
        } else {
            render_asset!(
                ui,
                ControlPanelDimensions::TAB_BUTTON_ROUTING_X,
                ControlPanelDimensions::TAB_BUTTON_Y,
                "RoutingNavButton/default/released@2x",
                0.5,
                0.0,
                Some(egui::Sense::click())
            )
        };

        let mixer_response = if self.current_tab == 1 {
            render_asset!(
                ui,
                ControlPanelDimensions::TAB_BUTTON_MIXER_X,
                ControlPanelDimensions::TAB_BUTTON_Y,
                "MixerNavButton/default/pressed@2x",
                0.5,
                0.0,
                Some(egui::Sense::click())
            )
        } else {
            render_asset!(
                ui,
                ControlPanelDimensions::TAB_BUTTON_MIXER_X,
                ControlPanelDimensions::TAB_BUTTON_Y,
                "MixerNavButton/default/released@2x",
                0.5,
                0.0,
                Some(egui::Sense::click())
            )
        };

        let ad_phase_response = if self.current_tab == 2 {
            render_asset!(
                ui,
                ControlPanelDimensions::TAB_BUTTON_AD_PHASE_X,
                ControlPanelDimensions::TAB_BUTTON_Y,
                "PhaseInvNavButton/default/pressed",
                1.0,
                0.0,
                Some(egui::Sense::click())
            )
        } else {
            render_asset!(
                ui,
                ControlPanelDimensions::TAB_BUTTON_AD_PHASE_X,
                ControlPanelDimensions::TAB_BUTTON_Y,
                "PhaseInvNavButton/default/released",
                1.0,
                0.0,
                Some(egui::Sense::click())
            )
        };

        if ControlPanelHelpers::mouse_down_on(&routing_response) {
            self.current_tab = 0;
        }

        if ControlPanelHelpers::mouse_down_on(&mixer_response) {
            self.current_tab = 1;
        }

        if ControlPanelHelpers::mouse_down_on(&ad_phase_response) {
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
            }
        }
    }

    fn draw_vu_meters(ui: &mut egui::Ui, left_vu_values: &[u8; 32], right_vu_values: &[u8; 32]) {
        for vu_meter_x in 0..32 {
            let pos_x = vu_meter_x * ControlPanelDimensions::VU_METER_X_STEP;

            let pos_x_left = ControlPanelDimensions::VU_METER_LEFT_X + pos_x;
            let pos_x_right = ControlPanelDimensions::VU_METER_RIGHT_X + pos_x;

            render_vu_meter!(
                ui,
                pos_x_left,
                ControlPanelDimensions::VU_METER_Y,
                left_vu_values[vu_meter_x],
                "MonitorPeakMeter/default/background"
            );

            render_vu_meter!(
                ui,
                pos_x_right,
                ControlPanelDimensions::VU_METER_Y,
                right_vu_values[vu_meter_x],
                "MonitorPeakMeter/default/background"
            );
        }
    }

    fn draw_vu_meter_comboboxes(&self, ui: &mut egui::Ui, left_vu_source: u8, right_vu_source: u8) {
        ControlPanelHelpers::draw_combobox(
            ui,
            ControlPanelDimensions::VU_SOURCE_LEFT_X,
            ControlPanelDimensions::VU_SOURCE_Y,
            ControlPanelDimensions::VU_SOURCE_WIDTH,
            left_vu_source,
            &Self::VU_SOURCE_SELECTIONS,
            |selected| {
                self.orion_hid
                    .lock()
                    .unwrap()
                    .queue_vu_source_left(selected);
            },
        );

        ControlPanelHelpers::draw_combobox(
            ui,
            ControlPanelDimensions::VU_SOURCE_RIGHT_X,
            ControlPanelDimensions::VU_SOURCE_Y,
            ControlPanelDimensions::VU_SOURCE_WIDTH,
            right_vu_source,
            &Self::VU_SOURCE_SELECTIONS,
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
            ControlPanelDimensions::CLOCK_CLOCK_SOURCE_X,
            ControlPanelDimensions::CLOCK_Y,
            ControlPanelDimensions::CLOCK_WIDTH,
            clock_source,
            &Self::CLOCK_SOURCE_SELECTIONS,
            |selected| {
                self.orion_hid.lock().unwrap().queue_clock_source(selected);
            },
        );

        ControlPanelHelpers::draw_combobox(
            ui,
            ControlPanelDimensions::CLOCK_SAMPLE_RATE_X,
            ControlPanelDimensions::CLOCK_Y,
            ControlPanelDimensions::CLOCK_WIDTH,
            sample_rate,
            &Self::SAMPLE_RATE_SELECTIONS,
            |selected| {
                self.orion_hid.lock().unwrap().queue_sample_rate(selected);
            },
        );
    }

    fn draw_line_out(&mut self, ui: &mut egui::Ui, mut line_out: u8) {
        line_out = self.line_out_ignore.value_with_ignore(line_out);

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
            false,
        );

        if should_write {
            self.line_out_ignore.begin_ignore(line_out);
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
                    self.line_out_ignore.begin_ignore(line_out_new);
                    self.orion_hid.lock().unwrap().queue_line_out(line_out_new);
                }
            }
        }
    }

    fn draw_serial_combobox(ui: &mut egui::Ui, serial: &[u8; 13]) {
        let serial_text = String::from_utf8_lossy(serial);

        ControlPanelHelpers::draw_combobox(
            ui,
            ControlPanelDimensions::SERIAL_X,
            ControlPanelDimensions::SERIAL_Y,
            ControlPanelDimensions::SERIAL_WIDTH,
            0,
            &[&serial_text],
            |_selected| {
                // TODO: Implement multi-interface control, and switch it here.
            },
        );
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
