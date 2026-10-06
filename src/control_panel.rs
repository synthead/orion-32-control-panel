// Hide console window on Windows.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;
use std::collections::BTreeSet;
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::Mutex;

use crate::orion_hid::OrionHid;
use crate::orion_state::OrionState;

struct ControlPanelRouteColors;

impl ControlPanelRouteColors {
    const ADC_DAC: egui::Color32 = egui::Color32::from_rgb(0x20, 0xa4, 0xf7);
    const USB: egui::Color32 = egui::Color32::from_rgb(0x00, 0x73, 0xff);
    const MADI: egui::Color32 = egui::Color32::from_rgb(0xff, 0xb8, 0x00);
    const ADAT: egui::Color32 = egui::Color32::from_rgb(0xff, 0x00, 0x37);
    const SPDIF: egui::Color32 = egui::Color32::from_rgb(0xd9, 0x24, 0x94);
    const MIX_LR: egui::Color32 = egui::Color32::from_rgb(0x72, 0xab, 0x20);
    const OSC_1: egui::Color32 = egui::Color32::from_rgb(0x38, 0x90, 0xa7);
    const OSC_2: egui::Color32 = egui::Color32::from_rgb(0x90, 0x38, 0xa7);
    const MUTE: egui::Color32 = egui::Color32::from_rgb(0x7b, 0x7b, 0x7b);
    const SELECT: egui::Color32 = egui::Color32::from_rgb(0x91, 0xb4, 0xdf);
}

struct ControlPanelRouteDimensions;

impl ControlPanelRouteDimensions {
    const START_X: f32 = 127.0;
    const START_Y_SOURCE: f32 = 284.0;
    const START_Y_TARGET: f32 = 448.0;

    const STEP_X: f32 = 30.0;
    const STEP_Y: f32 = 15.0;

    const SIZE_X: f32 = 29.0;
    const SIZE_Y: f32 = 14.0;
}

pub struct ControlPanel {
    pub orion_state: Arc<Mutex<OrionState>>,
    pub orion_hid: OrionHid,

    route_labels: [[String; 32]; 7],

    route_source_counts: [usize; 7],
    route_target_counts: [usize; 7],

    route_label_edit_cell: Option<[u8; 2]>,
    route_label_edit_string: String,

    route_source_select_range_row: Option<u8>,
    route_source_select_range_column: u8,

    route_source_select_toggle_columns: HashSet<u8>,

    route_source_select_row: u8,
    route_source_select_columns: BTreeSet<u8>,

    route_target_select_cell: Option<[u8; 2]>,

    route_source_mousedown: bool,
    route_target_hover_cell: Option<[u8; 2]>,
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
                        "../assets/images/",
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

    pub fn new(orion_state: Arc<Mutex<OrionState>>, orion_hid: OrionHid) -> Self {
        let route_labels = std::array::from_fn(|group| {
            let start_at = if group == 3 { 33 } else { 1 };

            std::array::from_fn(|i| (i + start_at).to_string())
        });

        Self {
            orion_state,
            orion_hid,

            route_labels,

            route_source_counts: [0; 7],
            route_target_counts: [0; 7],

            route_label_edit_cell: None,
            route_label_edit_string: String::new(),

            route_source_select_range_row: None,
            route_source_select_range_column: 0,

            route_source_select_toggle_columns: HashSet::new(),

            route_source_select_row: 0,
            route_source_select_columns: BTreeSet::new(),

            route_target_select_cell: None,

            route_source_mousedown: false,
            route_target_hover_cell: None,
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

                let madi_1_count = match orion_state.sample_rate {
                    5 | 6 => 16,
                    _ => 32,
                };

                let madi_2_count = match orion_state.sample_rate {
                    0 | 1 | 2 => 32,
                    _ => 0,
                };

                let adat_count = match orion_state.sample_rate {
                    3 | 4 => 8,
                    5 | 6 => 4,
                    _ => 16,
                };

                self.route_source_counts = [
                    32,
                    32,
                    madi_1_count.try_into().unwrap(),
                    madi_2_count.try_into().unwrap(),
                    adat_count.try_into().unwrap(),
                    2,
                    2,
                ];

                self.route_target_counts = [
                    32,
                    32,
                    madi_1_count.try_into().unwrap(),
                    madi_2_count.try_into().unwrap(),
                    adat_count.try_into().unwrap(),
                    2,
                    32,
                ];

                self.draw_route_sources(ui);
                self.draw_route_targets(ui, &orion_state.routes);

                self.draw_route_selections(ui);
                self.draw_route_drag(ui);

                self.clear_target_selection_when_outside(ui);
                self.update_route_targets_on_mouseup(ui, &orion_state.routes);

                if self.route_label_edit_cell.is_some() {
                    self.draw_route_editor(ui);
                }

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

    fn draw_route_sources(&mut self, ui: &mut egui::Ui) {
        for row in 0..self.route_source_counts.len() {
            for column in 0..self.route_source_counts[row] as u8 {
                let label = &self.route_labels[row][column as usize].clone();
                let color = Self::route_color_for_route(row as u8);

                let rect =
                    Self::draw_route_rect_with_label(ui, column, row as u8, true, color, label);

                let id = ui.id().with(("route_source", column, row as u8));
                let response = ui.interact(rect, id, egui::Sense::click());

                if response.hovered() && ui.input(|i| i.pointer.primary_pressed()) {
                    self.route_source_mousedown = true;

                    self.update_source_selections(ui, column, row as u8);
                }

                if response.double_clicked() {
                    self.route_label_edit_string = label.clone();
                    self.route_label_edit_cell = Some([column, row as u8]);
                }
            }
        }

        if ui.input(|i| i.pointer.any_pressed()) && !self.route_source_mousedown {
            self.clear_selections(ui);
        }
    }

    fn update_source_selections(&mut self, ui: &mut egui::Ui, column: u8, row: u8) {
        let shift_held = ui.input(|i| i.modifiers.shift);
        let ctrl_held = ui.input(|i| i.modifiers.ctrl);

        self.route_target_select_cell = None;

        if self.route_source_select_row != row {
            self.route_source_select_row = row;

            self.route_source_select_toggle_columns.clear();
            self.route_source_select_columns.clear();
        }

        'done: {
            if shift_held {
                self.route_source_select_columns.clear();

                if self.route_source_select_range_row == Some(row) {
                    self.route_source_select_columns.extend(
                        self.route_source_select_range_column.min(column)
                            ..=self.route_source_select_range_column.max(column),
                    );
                } else {
                    self.route_source_select_columns.insert(column);
                }

                break 'done;
            }

            self.route_source_select_range_row = Some(row);
            self.route_source_select_range_column = column;

            self.route_source_select_toggle_columns =
                self.route_source_select_columns.iter().copied().collect();

            if !ctrl_held {
                if self.route_source_select_toggle_columns.contains(&column) {
                    break 'done;
                }

                self.route_source_select_toggle_columns.clear();
            }

            self.route_source_select_columns.clear();

            if !self.route_source_select_toggle_columns.remove(&column) {
                self.route_source_select_toggle_columns.insert(column);
            }
        }

        self.route_source_select_columns
            .extend(self.route_source_select_toggle_columns.iter().copied());
    }

    fn draw_route_targets(&mut self, ui: &mut egui::Ui, routes: &[[u8; 32]; 7]) {
        self.route_target_hover_cell = None;

        for row in 0..self.route_target_counts.len() {
            for column in 0..self.route_target_counts[row] {
                let route = routes[row][column];
                let color = Self::route_color_for_route(route);
                let label = &self.route_label_for_route(route);

                let rect = Self::draw_route_rect_with_label(
                    ui,
                    column as u8,
                    row as u8,
                    false,
                    color,
                    label,
                );

                let id = ui.id().with(("route_target", column, row as u8));
                let response = ui.interact(rect, id, egui::Sense::click());

                if response.contains_pointer() {
                    self.route_target_hover_cell = Some([column as u8, row as u8]);

                    if ui.input(|i| i.pointer.primary_pressed()) {
                        self.update_target_selections(ui, column as u8, row as u8, route);
                    }
                }
            }
        }
    }

    fn update_target_selections(&mut self, ui: &mut egui::Ui, column: u8, row: u8, route: u8) {
        self.route_target_select_cell = Some([column, row]);

        self.route_source_select_toggle_columns.clear();
        self.route_source_select_columns.clear();

        if let Some([source_column, source_row]) = Self::route_source_cell_for_route(route) {
            self.route_source_select_row = source_row;
            self.route_source_select_columns.insert(source_column);
        }
    }

    fn draw_route_selection(&self, ui: &mut egui::Ui, column: u8, row: u8, is_source: bool) {
        let label = &self.route_labels[row as usize][column as usize];

        Self::draw_route_rect_with_label(
            ui,
            column,
            row,
            is_source,
            ControlPanelRouteColors::SELECT,
            label,
        );
    }

    fn draw_route_selections(&self, ui: &mut egui::Ui) {
        for &column in &self.route_source_select_columns {
            self.draw_route_selection(ui, column, self.route_source_select_row, true);
        }

        if let Some([target_column, target_row]) = self.route_target_select_cell {
            self.draw_route_selection(ui, target_column, target_row, false);
        }
    }

    fn clear_target_selection_when_outside(&mut self, ui: &mut egui::Ui) {
        let rect = egui::Rect::from_min_max(egui::pos2(127.0, 448.0), egui::pos2(1086.0, 552.0));
        let hover = ui.rect_contains_pointer(rect);

        if self.route_target_select_cell.is_some() && !hover {
            self.clear_selections(ui);
        }
    }

    fn update_route_targets_on_mouseup(&mut self, ui: &mut egui::Ui, routes: &[[u8; 32]; 7]) {
        if self.route_source_mousedown && ui.input(|i| i.pointer.primary_released()) {
            if let Some([target_column, target_row]) = self.route_target_hover_cell {
                let mut new_routes = routes[target_row as usize];

                for (column_index, &source_column) in self.route_source_select_columns.iter().enumerate() {
                    let route = source_column * 0x08 + self.route_source_select_row;

                    new_routes[target_column as usize + column_index] = route;
                }


                self.orion_hid.queue_routes(target_row, &new_routes);
            }

            self.route_source_mousedown = false;
        }
    }

    fn clear_selections(&mut self, ui: &mut egui::Ui) {
        self.route_target_select_cell = None;
        self.route_source_select_columns.clear();
    }

    fn draw_route_drag(&mut self, ui: &mut egui::Ui) {
        if !self.route_source_mousedown || self.route_target_hover_cell.is_none() {
            return;
        }

        let [target_column, target_row] = self.route_target_hover_cell.unwrap();

        for (column_index, &source_column) in self.route_source_select_columns.iter().enumerate() {
            let label =
                &self.route_labels[self.route_source_select_row as usize][source_column as usize];
            let color = Self::route_color_for_route(self.route_source_select_row);

            Self::draw_route_rect_with_label(
                ui,
                target_column + column_index as u8,
                target_row,
                false,
                color,
                label,
            );
        }
    }

    fn draw_route_editor(&mut self, ui: &mut egui::Ui) {
        let [column, row] = self.route_label_edit_cell.unwrap();
        let (pos_x, pos_y) = Self::route_cell_pos(column, row, true);
        let color = Self::route_color_for_route(row);

        let rect = egui::Rect::from_min_size(
            egui::pos2(pos_x - 14.0, pos_y - 2.0),
            egui::vec2(58.0, 18.0),
        );

        ui.painter().rect(
            rect,
            egui::CornerRadius::same(5),
            color,
            egui::Stroke::new(1.0, egui::Color32::BLACK),
            egui::StrokeKind::Inside,
        );

        let hitbox_rect = egui::Rect::from_min_size(
            egui::pos2(pos_x, pos_y),
            egui::vec2(
                ControlPanelRouteDimensions::SIZE_X,
                ControlPanelRouteDimensions::SIZE_Y,
            ),
        );

        if ui.input(|i| {
            i.pointer.any_click()
                && i.pointer
                    .interact_pos()
                    .is_some_and(|pos| !hitbox_rect.contains(pos))
        }) {
            self.route_labels[row as usize][column as usize] = self.route_label_edit_string.clone();
            self.route_label_edit_cell = None;
        }

        let text_edit_id = ui.id().with(("label_route_edit", column, row));

        if !ui.memory(|mem| mem.has_focus(text_edit_id)) {
            let mut state = egui::widgets::text_edit::TextEditState::load(ui.ctx(), text_edit_id)
                .unwrap_or_default();

            let label_chars = self.route_label_edit_string.chars().count();

            state
                .cursor
                .set_char_range(Some(egui::text::CCursorRange::two(
                    egui::text::CCursor::new(0),
                    egui::text::CCursor::new(label_chars),
                )));

            state.store(ui.ctx(), text_edit_id);

            ui.memory_mut(|mem| mem.request_focus(text_edit_id));
        }

        ui.put(
            rect,
            egui::TextEdit::singleline(&mut self.route_label_edit_string)
                .id(text_edit_id)
                .font(egui::FontId::proportional(12.0))
                .horizontal_align(egui::Align::Center)
                .vertical_align(egui::Align::Center)
                .text_color(egui::Color32::WHITE)
                .frame(egui::Frame::NONE)
                .event_filter(egui::EventFilter {
                    tab: true,
                    ..Default::default()
                }),
        );

        if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            self.route_labels[row as usize][column as usize] = self.route_label_edit_string.clone();
            self.route_label_edit_cell = None;
        }

        if ui.input(|i| i.key_pressed(egui::Key::Tab)) {
            self.route_labels[row as usize][column as usize] = self.route_label_edit_string.clone();

            let mut column_next = column as usize;
            let mut row_next = row as usize;

            column_next += 1;

            if column_next >= self.route_source_counts[row_next] {
                column_next = 0;
                row_next += 1;
            }

            if row_next >= self.route_source_counts.len() {
                row_next = 0;
            }

            self.route_label_edit_string = self.route_labels[row_next][column_next].clone();
            self.route_label_edit_cell = Some([column_next as u8, row_next as u8]);
        }

        if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.route_label_edit_cell = None;
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

    fn route_source_cell_for_route(route: u8) -> Option<[u8; 2]> {
        match route {
            0x07 | 0x0f | 0x17 => None,

            _ => {
                let row = route % 0x08;
                let column = route / 0x08;

                Some([column, row])
            }
        }
    }

    fn route_label_for_route(&self, route: u8) -> String {
        match route {
            0x07 => "O1".to_string(),
            0x0f => "O2".to_string(),
            0x17 => "M".to_string(),

            _ => {
                let route_type = route % 0x08;
                let route_index = route / 0x08;

                self.route_labels[route_type as usize][route_index as usize].clone()
            }
        }
    }

    fn route_color_for_route(route: u8) -> egui::Color32 {
        match route {
            0x07 => ControlPanelRouteColors::OSC_1,
            0x0f => ControlPanelRouteColors::OSC_2,
            0x17 => ControlPanelRouteColors::MUTE,

            _ => match route % 0x08 {
                0x00 => ControlPanelRouteColors::ADC_DAC,
                0x01 => ControlPanelRouteColors::USB,
                0x02 => ControlPanelRouteColors::MADI,
                0x03 => ControlPanelRouteColors::MADI,
                0x04 => ControlPanelRouteColors::ADAT,
                0x05 => ControlPanelRouteColors::SPDIF,
                0x06 => ControlPanelRouteColors::MIX_LR,

                _ => egui::Color32::BLACK,
            },
        }
    }

    fn route_cell_pos(column: u8, row: u8, is_source: bool) -> (f32, f32) {
        let base_pos_y = if is_source {
            ControlPanelRouteDimensions::START_Y_SOURCE
        } else {
            ControlPanelRouteDimensions::START_Y_TARGET
        };

        let pos_x = ControlPanelRouteDimensions::START_X
            + column as f32 * ControlPanelRouteDimensions::STEP_X;
        let pos_y = base_pos_y + row as f32 * ControlPanelRouteDimensions::STEP_Y;

        (pos_x, pos_y)
    }

    fn draw_route_rect_with_label(
        ui: &mut egui::Ui,
        column: u8,
        row: u8,
        is_source: bool,
        color: egui::Color32,
        label: &String,
    ) -> egui::Rect {
        let (pos_x, pos_y) = Self::route_cell_pos(column, row, is_source);

        let rect = egui::Rect::from_min_size(
            egui::pos2(pos_x, pos_y),
            egui::vec2(
                ControlPanelRouteDimensions::SIZE_X,
                ControlPanelRouteDimensions::SIZE_Y,
            ),
        );

        ui.painter().rect_filled(rect, 0.0, color);

        Self::draw_text(
            ui,
            pos_x as u16 + 15,
            pos_y as u16,
            (0xff, 0xff, 0xff),
            label,
        );

        rect
    }
}

impl eframe::App for ControlPanel {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.ui_content(ui);
    }
}
