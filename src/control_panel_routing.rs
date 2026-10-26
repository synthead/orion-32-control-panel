use std::collections::BTreeSet;
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::Mutex;

use crate::routing_constants::RoutingCodes;

use crate::orion_file::OrionFile;
use crate::orion_hid::OrionHid;
use crate::orion_state::OrionState;

use crate::control_panel_helpers::ControlPanelHelpers;
use crate::text_editor::TextEditor;

use crate::render_asset;

struct ControlPanelRoutingColors;

impl ControlPanelRoutingColors {
    const SELECT: egui::Color32 = egui::Color32::from_rgb(0x91, 0xb4, 0xdf);
    const TEXT: egui::Color32 = egui::Color32::WHITE;
}

struct ControlPanelRoutingDimensions;

impl ControlPanelRoutingDimensions {
    const BACKGROUND_X: u16 = 0;
    const BACKGROUND_Y: u16 = 226;

    const START_X: f32 = 127.0;
    const START_Y_SOURCE: f32 = 284.0;
    const START_Y_TARGET: f32 = 448.0;

    const STEP_X: f32 = 30.0;
    const STEP_Y: f32 = 15.0;

    const SIZE_X: f32 = 29.0;
    const SIZE_Y: f32 = 14.0;

    const TEXT_X: f32 = 128.0;
    const TEXT_Y: f32 = 409.0;
    const TEXT_SIZE: f32 = 10.5;

    const PRESENSE_X: u16 = 1095;
    const PRESENSE_ADAT_IN_2_X: u16 = 1112;
    const PRESENSE_USB_PLAY_Y: u16 = 301;
    const PRESENSE_MADI_IN_Y: u16 = 323;
    const PRESENSE_ADAT_IN_Y: u16 = 346;
    const PRESENSE_SPDIF_IN_Y: u16 = 361;

    const NAV_X: u16 = 1106;
    const NAV_UNDO_Y: u16 = 410;
    const NAV_REDO_Y: u16 = 436;
    const NAV_SAVE_Y: u16 = 462;
    const NAV_LOAD_Y: u16 = 488;

    const HOWTO_X: u16 = 1094;
    const HOWTO_Y: u16 = 652;
}

pub(crate) struct ControlPanelRouting {
    orion_state: Arc<Mutex<OrionState>>,
    orion_hid: Arc<Mutex<OrionHid>>,

    text_editor: Arc<Mutex<TextEditor>>,

    config_dir: Option<std::path::PathBuf>,

    undo_buffer: Vec<(u8, [u8; 32], [u8; 32])>,
    undo_cursor: usize,

    source_counts: [usize; 7],
    target_counts: [usize; 7],

    label_edit_cell: Option<[u8; 2]>,
    source_select_range_row: Option<u8>,
    source_select_range_column: u8,

    source_select_toggle_columns: HashSet<u8>,

    source_select_row: u8,
    source_select_columns: BTreeSet<u8>,

    target_select_cell: Option<[u8; 2]>,

    source_mousedown: bool,
    target_hover_cell: Option<[u8; 2]>,
}

impl ControlPanelRouting {
    const CONFIG_DIR: &str = "Orion 32 Control Panel";

    const SOURCE_ROW_LABELS: [&str; 7] = [
        "ADC",
        "USB PLAY",
        "MADI IN 1-32",
        "MADI IN 33-64",
        "ADAT IN",
        "S/PDIF IN",
        "MIX 1 L/R",
    ];

    pub(crate) fn new(
        orion_state: Arc<Mutex<OrionState>>,
        orion_hid: Arc<Mutex<OrionHid>>,
        text_editor: Arc<Mutex<TextEditor>>,
    ) -> Self {
        let config_dir = directories::BaseDirs::new()
            .map(|dirs| dirs.config_dir().join(ControlPanelRouting::CONFIG_DIR));

        Self {
            orion_state,
            orion_hid,

            config_dir,

            text_editor,

            undo_buffer: Vec::new(),
            undo_cursor: 0,

            source_counts: [0; 7],
            target_counts: [0; 7],

            label_edit_cell: None,

            source_select_range_row: None,
            source_select_range_column: 0,

            source_select_toggle_columns: HashSet::new(),

            source_select_row: 0,
            source_select_columns: BTreeSet::new(),

            target_select_cell: None,

            source_mousedown: false,
            target_hover_cell: None,
        }
    }

    pub(crate) fn draw(&mut self, ui: &mut egui::Ui) {
        let mut orion_state = self.orion_state.lock().unwrap().clone();

        orion_state.calculate_channel_counts();
        self.source_counts = orion_state.source_counts;
        self.target_counts = orion_state.target_counts;

        Self::draw_base_graphics(ui);
        self.draw_undo(ui);
        self.draw_save(ui);

        self.draw_sources(ui, &orion_state);
        self.draw_targets(ui, &orion_state);

        self.draw_drag(ui, &orion_state);
        self.draw_text(ui, &orion_state.labels);

        self.draw_signals(ui, orion_state.signals);

        self.clear_target_selection_when_outside(ui);
        self.update_targets_on_mouseup(ui, &orion_state.routes);
    }

    pub(crate) fn create_config_dir(&self) {
        if let Some(config_dir) = &self.config_dir {
            std::fs::create_dir_all(config_dir).unwrap_or_else(|error| {
                panic!(
                    "Failed to create config directory '{}': {error}",
                    config_dir.display()
                )
            });
        }
    }

    fn draw_base_graphics(ui: &mut egui::Ui) {
        render_asset!(
            ui,
            ControlPanelRoutingDimensions::BACKGROUND_X,
            ControlPanelRoutingDimensions::BACKGROUND_Y,
            "Routing/default/background",
            1.0,
            0.0,
            None
        );

        render_asset!(
            ui,
            ControlPanelRoutingDimensions::HOWTO_X,
            ControlPanelRoutingDimensions::HOWTO_Y,
            "HowtoButton/default/released",
            1.0,
            0.0,
            None
        );
    }

    fn draw_undo(&mut self, ui: &mut egui::Ui) {
        let undo_response = render_asset!(
            ui,
            ControlPanelRoutingDimensions::NAV_X,
            ControlPanelRoutingDimensions::NAV_UNDO_Y,
            "UndoButton/default/released",
            1.0,
            0.0,
            Some(egui::Sense::click())
        );

        if undo_response.is_pointer_button_down_on() != undo_response.hovered() {
            render_asset!(
                ui,
                ControlPanelRoutingDimensions::NAV_X,
                ControlPanelRoutingDimensions::NAV_UNDO_Y,
                "UndoButton/default/pressed",
                1.0,
                0.0,
                None
            );
        }

        let redo_response = render_asset!(
            ui,
            ControlPanelRoutingDimensions::NAV_X,
            ControlPanelRoutingDimensions::NAV_REDO_Y,
            "RedoButton/default/released",
            1.0,
            0.0,
            Some(egui::Sense::click())
        );

        if redo_response.is_pointer_button_down_on() != redo_response.hovered() {
            render_asset!(
                ui,
                ControlPanelRoutingDimensions::NAV_X,
                ControlPanelRoutingDimensions::NAV_REDO_Y,
                "RedoButton/default/pressed",
                1.0,
                0.0,
                None
            );
        }

        if undo_response.clicked() {
            self.queue_routes_undo();
        }

        if redo_response.clicked() {
            self.queue_routes_redo();
        }
    }

    fn draw_save(&mut self, ui: &mut egui::Ui) {
        let save_response = render_asset!(
            ui,
            ControlPanelRoutingDimensions::NAV_X,
            ControlPanelRoutingDimensions::NAV_SAVE_Y,
            "SaveRoutingButton/default/released",
            1.0,
            0.0,
            Some(egui::Sense::click())
        );

        if save_response.is_pointer_button_down_on() != save_response.hovered() {
            render_asset!(
                ui,
                ControlPanelRoutingDimensions::NAV_X,
                ControlPanelRoutingDimensions::NAV_SAVE_Y,
                "SaveRoutingButton/default/pressed",
                1.0,
                0.0,
                None
            );
        }

        let load_response = render_asset!(
            ui,
            ControlPanelRoutingDimensions::NAV_X,
            ControlPanelRoutingDimensions::NAV_LOAD_Y,
            "LoadRoutingButton/default/released",
            1.0,
            0.0,
            Some(egui::Sense::click())
        );

        if load_response.is_pointer_button_down_on() != load_response.hovered() {
            render_asset!(
                ui,
                ControlPanelRoutingDimensions::NAV_X,
                ControlPanelRoutingDimensions::NAV_LOAD_Y,
                "LoadRoutingButton/default/pressed",
                1.0,
                0.0,
                None
            );
        }

        if save_response.clicked() {
            let mut dialog = rfd::FileDialog::new().add_filter("Orion 32 Routing File", &["o32r"]);

            if let Some(config_dir) = &self.config_dir {
                dialog = dialog.set_directory(config_dir);
            }

            if let Some(file) = dialog.save_file() {
                let orion_file = OrionFile::new(file);
                let orion_state = self.orion_state.lock().unwrap().clone();

                orion_file.encode(
                    &orion_state.labels,
                    &orion_state.routes,
                    &self.source_counts,
                    &self.target_counts,
                );
            }
        }

        if load_response.clicked() {
            let mut dialog = rfd::FileDialog::new().add_filter("Orion 32 Routing File", &["o32r"]);

            if let Some(config_dir) = &self.config_dir {
                dialog = dialog.set_directory(config_dir);
            }

            if let Some(file) = dialog.pick_file() {
                let orion_file = OrionFile::new(file);
                let (labels, routes) = orion_file.decode();

                {
                    let mut orion_state = self.orion_state.lock().unwrap();

                    orion_state.labels = labels;
                }

                {
                    let mut orion_hid = self.orion_hid.lock().unwrap();

                    for row in 0..7 {
                        orion_hid.queue_routes(row as u8, &routes[row]);
                    }
                }

                self.clear_undo();
            }
        }
    }

    fn draw_source(&mut self, ui: &mut egui::Ui, column: u8, row: u8, orion_state: &OrionState) {
        let label = &orion_state.labels[row as usize][column as usize].clone();
        let (pos_x, pos_y) = Self::cell_pos(column, row, true);

        let color = if self.source_select_row == row && self.source_select_columns.contains(&column)
        {
            ControlPanelRoutingColors::SELECT
        } else {
            orion_state.color_for_route(row)
        };

        let rect = egui::Rect::from_min_size(
            egui::pos2(pos_x, pos_y),
            egui::vec2(
                ControlPanelRoutingDimensions::SIZE_X,
                ControlPanelRoutingDimensions::SIZE_Y,
            ),
        );

        ui.painter().rect_filled(rect, 0.0, color);

        let id_with = format!("route_{column}_{row}");
        let id = ui.id().with(&id_with);
        let response = ui.interact(rect, id, egui::Sense::click());

        if response.double_clicked() {
            self.label_edit_cell = Some([column, row]);
        }

        if response.hovered() && ui.input(|i| i.pointer.primary_pressed()) {
            self.source_mousedown = true;
            self.update_source_selections(ui, column, row);
        }

        self.text_editor.lock().unwrap().draw(
            ui,
            rect,
            &id_with,
            &label,
            ControlPanelRoutingDimensions::TEXT_SIZE,
            ControlPanelRoutingColors::TEXT,
        );
    }

    fn draw_source_editing(
        &mut self,
        ui: &mut egui::Ui,
        column: u8,
        row: u8,
        orion_state: &OrionState,
    ) {
        let label = &orion_state.labels[row as usize][column as usize].clone();
        let (pos_x, pos_y) = Self::cell_pos(column, row, true);
        let color = orion_state.color_for_route(row);

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

        let level_edit_result = self.text_editor.lock().unwrap().draw(
            ui,
            rect,
            &format!("route_{column}_{row}"),
            &label,
            ControlPanelRoutingDimensions::TEXT_SIZE,
            ControlPanelRoutingColors::TEXT,
        );

        if let Some((route_text, tab)) = level_edit_result {
            let mut orion_state = self.orion_state.lock().unwrap();

            orion_state.labels[row as usize][column as usize] = route_text;

            if let Some(tab) = tab {
                let mut column_next = column as i16 + tab as i16;
                let mut row_next = row as usize;

                if column_next < 0 {
                    row_next = (row_next + self.source_counts.len() - 1) % self.source_counts.len();
                    column_next = self.source_counts[row_next] as i16 - 1;
                } else if column_next == self.source_counts[row_next] as i16 {
                    row_next = (row_next + 1) % self.source_counts.len();
                    column_next = 0;
                }

                self.label_edit_cell = Some([column_next as u8, row_next as u8]);

                self.text_editor
                    .lock()
                    .unwrap()
                    .set_editing_id(&format!("route_{column_next}_{row_next}"));
            } else {
                self.label_edit_cell = None;
            }
        }
    }

    fn draw_sources(&mut self, ui: &mut egui::Ui, orion_state: &OrionState) {
        for row in 0..self.source_counts.len() as u8 {
            for column in 0..self.source_counts[row as usize] as u8 {
                if Some([column, row]) != self.label_edit_cell {
                    self.draw_source(ui, column, row, &orion_state);
                }
            }
        }

        if let Some([column, row]) = self.label_edit_cell {
            self.draw_source_editing(ui, column, row, &orion_state);
        }
    }

    fn update_source_selections(&mut self, ui: &mut egui::Ui, column: u8, row: u8) {
        let shift_held = ui.input(|i| i.modifiers.shift);
        let ctrl_held = ui.input(|i| i.modifiers.ctrl);

        self.target_select_cell = None;

        if self.source_select_row != row {
            self.source_select_row = row;

            self.source_select_toggle_columns.clear();
            self.source_select_columns.clear();
        }

        'done: {
            if shift_held {
                self.source_select_columns.clear();

                if self.source_select_range_row == Some(row) {
                    self.source_select_columns.extend(
                        self.source_select_range_column.min(column)
                            ..=self.source_select_range_column.max(column),
                    );
                } else {
                    self.source_select_columns.insert(column);
                }

                break 'done;
            }

            self.source_select_range_row = Some(row);
            self.source_select_range_column = column;

            self.source_select_toggle_columns =
                self.source_select_columns.iter().copied().collect();

            if !ctrl_held {
                if self.source_select_toggle_columns.contains(&column) {
                    break 'done;
                }

                self.source_select_toggle_columns.clear();
            }

            self.source_select_columns.clear();

            if !self.source_select_toggle_columns.remove(&column) {
                self.source_select_toggle_columns.insert(column);
            }
        }

        self.source_select_columns
            .extend(self.source_select_toggle_columns.iter().copied());
    }

    fn draw_targets(&mut self, ui: &mut egui::Ui, orion_state: &OrionState) {
        self.target_hover_cell = None;

        for row in 0..self.target_counts.len() {
            for column in 0..self.target_counts[row] {
                let routes = &orion_state.routes[row];
                let route = routes[column];

                let color = if self.target_select_cell == Some([column as u8, row as u8]) {
                    ControlPanelRoutingColors::SELECT
                } else {
                    orion_state.color_for_route(route)
                };

                let label = &orion_state.label_for_route(route);

                let rect =
                    Self::draw_rect_with_label(ui, column as u8, row as u8, false, color, label);

                let id = ui.id().with(("route_target", column, row));
                let response = ui.interact(rect, id, egui::Sense::click());

                if response.contains_pointer() {
                    self.target_hover_cell = Some([column as u8, row as u8]);

                    if ControlPanelHelpers::mouse_down_on(&response) {
                        self.update_target_selections(column as u8, row as u8, route);
                    }
                }

                egui::Popup::context_menu(&response).show(|ui| {
                    if ui.button("Mute").clicked() {
                        self.update_targets_from_context_menu(
                            &routes,
                            Some(column),
                            row,
                            RoutingCodes::MUTE,
                        );

                        ui.close();
                    }

                    if ui.button("Mute Row").clicked() {
                        self.update_targets_from_context_menu(
                            &routes,
                            None,
                            row,
                            RoutingCodes::MUTE,
                        );

                        ui.close();
                    }

                    if ui.button("Oscillator1").clicked() {
                        self.update_targets_from_context_menu(
                            &routes,
                            Some(column),
                            row,
                            RoutingCodes::OSC_1,
                        );

                        ui.close();
                    }

                    if ui.button("Oscillator2").clicked() {
                        self.update_targets_from_context_menu(
                            &routes,
                            Some(column),
                            row,
                            RoutingCodes::OSC_2,
                        );

                        ui.close();
                    }
                });
            }
        }
    }

    fn update_targets_from_context_menu(
        &mut self,
        old_routes: &[u8; 32],
        column: Option<usize>,
        row: usize,
        route: u8,
    ) {
        let mut new_routes = *old_routes;

        if let Some(column) = column {
            new_routes[column] = route;
        } else {
            for column in 0..self.target_counts[row] {
                new_routes[column] = route;
            }
        }

        self.queue_routes_with_undo(row as u8, &old_routes, &new_routes);
    }

    fn update_target_selections(&mut self, column: u8, row: u8, route: u8) {
        self.target_select_cell = Some([column, row]);

        self.source_select_toggle_columns.clear();
        self.source_select_columns.clear();

        if let Some([source_column, source_row]) = self.source_cell_for_route(route) {
            self.source_select_row = source_row;
            self.source_select_columns.insert(source_column);
        }
    }

    fn draw_text(&self, ui: &mut egui::Ui, labels: &[[String; 32]; 7]) {
        if self.source_select_columns.is_empty() {
            return;
        }

        let text = format!(
            "{}: {}",
            Self::SOURCE_ROW_LABELS[self.source_select_row as usize],
            self.source_select_columns
                .iter()
                .map(|&column| labels[self.source_select_row as usize][column as usize].as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );

        ui.painter().text(
            egui::pos2(
                ControlPanelRoutingDimensions::TEXT_X,
                ControlPanelRoutingDimensions::TEXT_Y,
            ),
            egui::Align2::LEFT_TOP,
            text,
            egui::FontId::proportional(ControlPanelRoutingDimensions::TEXT_SIZE),
            ControlPanelRoutingColors::TEXT,
        );
    }

    fn draw_signals(&self, ui: &mut egui::Ui, signals: u8) {
        // TODO: Figure out the rest of these signals.
        if signals == 0x09 {
            render_asset!(
                ui,
                ControlPanelRoutingDimensions::PRESENSE_X,
                ControlPanelRoutingDimensions::PRESENSE_USB_PLAY_Y,
                "PresenceLed/default/pressed",
                1.0,
                0.0,
                None
            );
        } else {
            render_asset!(
                ui,
                ControlPanelRoutingDimensions::PRESENSE_X,
                ControlPanelRoutingDimensions::PRESENSE_USB_PLAY_Y,
                "PresenceLed/default/released",
                1.0,
                0.0,
                None
            );
        }

        render_asset!(
            ui,
            ControlPanelRoutingDimensions::PRESENSE_X,
            ControlPanelRoutingDimensions::PRESENSE_MADI_IN_Y,
            "PresenceLed/default/released",
            1.0,
            0.0,
            None
        );

        render_asset!(
            ui,
            ControlPanelRoutingDimensions::PRESENSE_X,
            ControlPanelRoutingDimensions::PRESENSE_ADAT_IN_Y,
            "PresenceLed/default/released",
            1.0,
            0.0,
            None
        );

        render_asset!(
            ui,
            ControlPanelRoutingDimensions::PRESENSE_ADAT_IN_2_X,
            ControlPanelRoutingDimensions::PRESENSE_ADAT_IN_Y,
            "PresenceLed/default/released",
            1.0,
            0.0,
            None
        );

        render_asset!(
            ui,
            ControlPanelRoutingDimensions::PRESENSE_X,
            ControlPanelRoutingDimensions::PRESENSE_SPDIF_IN_Y,
            "PresenceLed/default/released",
            1.0,
            0.0,
            None
        );
    }

    fn clear_target_selection_when_outside(&mut self, ui: &mut egui::Ui) {
        let rect = egui::Rect::from_min_max(egui::pos2(127.0, 448.0), egui::pos2(1086.0, 552.0));
        let hover = ui.rect_contains_pointer(rect);

        if self.target_select_cell.is_some() && !hover {
            self.clear_selections();
        }
    }

    fn update_targets_on_mouseup(&mut self, ui: &mut egui::Ui, routes: &[[u8; 32]; 7]) {
        if self.source_mousedown && ui.input(|i| i.pointer.primary_released()) {
            if let Some([target_column, target_row]) = self.target_hover_cell {
                let old_routes = routes[target_row as usize];
                let mut new_routes = old_routes;

                let target_row_count = self.target_counts[target_row as usize];

                for (column_index, &source_column) in self.source_select_columns.iter().enumerate()
                {
                    if target_column as usize + column_index >= target_row_count {
                        break;
                    }

                    let route = source_column * RoutingCodes::COLUMN + self.source_select_row;

                    new_routes[target_column as usize + column_index] = route;
                }

                self.queue_routes_with_undo(target_row, &old_routes, &new_routes);
            }

            self.source_mousedown = false;
        }
    }

    fn clear_selections(&mut self) {
        self.target_select_cell = None;
        self.source_select_columns.clear();
    }

    fn draw_drag(&mut self, ui: &mut egui::Ui, orion_state: &OrionState) {
        if !self.source_mousedown || self.target_hover_cell.is_none() {
            return;
        }

        let [target_column, target_row] = self.target_hover_cell.unwrap();
        let target_row_count = orion_state.target_counts[target_row as usize];

        for (column_index, &source_column) in self.source_select_columns.iter().enumerate() {
            if target_column as usize + column_index >= target_row_count {
                return;
            }

            let label =
                &orion_state.labels[self.source_select_row as usize][source_column as usize];
            let color = orion_state.color_for_route(self.source_select_row);

            Self::draw_rect_with_label(
                ui,
                target_column + column_index as u8,
                target_row,
                false,
                color,
                label,
            );
        }
    }

    fn source_cell_for_route(&self, route: u8) -> Option<[u8; 2]> {
        match route {
            RoutingCodes::OSC_1 | RoutingCodes::OSC_2 | RoutingCodes::MUTE => None,

            _ => {
                let row = route % RoutingCodes::COLUMN;
                let column = route / RoutingCodes::COLUMN;

                if row == RoutingCodes::USB && column as usize >= self.source_counts[1] {
                    return None;
                }

                Some([column, row])
            }
        }
    }

    fn cell_pos(column: u8, row: u8, is_source: bool) -> (f32, f32) {
        let base_pos_y = if is_source {
            ControlPanelRoutingDimensions::START_Y_SOURCE
        } else {
            ControlPanelRoutingDimensions::START_Y_TARGET
        };

        let pos_x = ControlPanelRoutingDimensions::START_X
            + column as f32 * ControlPanelRoutingDimensions::STEP_X;
        let pos_y = base_pos_y + row as f32 * ControlPanelRoutingDimensions::STEP_Y;

        (pos_x, pos_y)
    }

    fn draw_rect_with_label(
        ui: &mut egui::Ui,
        column: u8,
        row: u8,
        is_source: bool,
        color: egui::Color32,
        label: &String,
    ) -> egui::Rect {
        let (pos_x, pos_y) = Self::cell_pos(column, row, is_source);

        let rect = egui::Rect::from_min_size(
            egui::pos2(pos_x, pos_y),
            egui::vec2(
                ControlPanelRoutingDimensions::SIZE_X,
                ControlPanelRoutingDimensions::SIZE_Y,
            ),
        );

        ui.painter().rect_filled(rect, 0.0, color);

        ui.painter().text(
            egui::pos2(pos_x + 15.0, pos_y),
            egui::Align2::CENTER_TOP,
            label,
            egui::FontId::proportional(ControlPanelRoutingDimensions::TEXT_SIZE),
            ControlPanelRoutingColors::TEXT,
        );

        rect
    }

    fn queue_routes_with_undo(&mut self, row: u8, old_routes: &[u8; 32], new_routes: &[u8; 32]) {
        self.undo_buffer.truncate(self.undo_cursor);
        self.undo_buffer.push((row, *old_routes, *new_routes));

        self.undo_cursor = self.undo_buffer.len();

        self.orion_hid.lock().unwrap().queue_routes(row, new_routes);
    }

    fn queue_routes_undo(&mut self) {
        if self.undo_cursor == 0 {
            return;
        }

        self.undo_cursor -= 1;

        let (row, old_routes, _) = self.undo_buffer[self.undo_cursor];

        self.orion_hid
            .lock()
            .unwrap()
            .queue_routes(row, &old_routes);
    }

    fn queue_routes_redo(&mut self) {
        if self.undo_cursor >= self.undo_buffer.len() {
            return;
        }

        let (row, _, new_routes) = self.undo_buffer[self.undo_cursor];

        self.orion_hid
            .lock()
            .unwrap()
            .queue_routes(row, &new_routes);

        self.undo_cursor += 1;
    }

    fn clear_undo(&mut self) {
        self.undo_buffer = Vec::new();
        self.undo_cursor = 0;
    }
}
