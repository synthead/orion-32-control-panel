use std::collections::BTreeSet;
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::mpsc;

use crate::constants::{
    RoutingColumns, RoutingLabelColors, RoutingLabels, RoutingRows, WidgetColors, WidgetDimensions,
};

use crate::orion_file::OrionFile;
use crate::orion_paths::OrionPaths;
use crate::orion_persistence::OrionPersistence;

use crate::orion_hid::OrionHid;
use crate::orion_state::{OrionState, OrionStateSignals, OrionStates};

use crate::control_panel_helpers::ControlPanelHelpers;
use crate::text_editor::TextEditor;

use crate::render_asset;

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

    const HITBOX_SIZE_X: f32 = 30.0;
    const HITBOX_SIZE_Y: f32 = 15.0;

    const EDIT_SIZE_X: f32 = 58.0;
    const EDIT_SIZE_Y: f32 = 18.0;

    const EDIT_SIZE_OFFSET_X: f32 = -14.0;
    const EDIT_SIZE_OFFSET_Y: f32 = -2.0;

    const TEXT_X: f32 = 138.0;
    const TEXT_Y: f32 = 412.0;

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

    const TARGET_ROUTES_MIN_X: f32 = 127.0;
    const TARGET_ROUTES_MIN_Y: f32 = 448.0;
    const TARGET_ROUTES_MAX_X: f32 = 1086.0;
    const TARGET_ROUTES_MAX_Y: f32 = 552.0;
}

type RoutingFileData = ([[String; 32]; 7], [[u8; 32]; 7]);

pub(crate) struct ControlPanelRouting {
    orion_states: Arc<Mutex<OrionStates>>,
    orion_hid: Arc<Mutex<OrionHid>>,
    orion_paths: Arc<Mutex<OrionPaths>>,
    orion_persistence: Arc<Mutex<OrionPersistence>>,

    text_editor: Arc<Mutex<TextEditor>>,

    file_dialog_tx: mpsc::Sender<Option<RoutingFileData>>,
    file_dialog_rx: mpsc::Receiver<Option<RoutingFileData>>,

    undo_buffer: Vec<(u8, [u8; 32], [u8; 32])>,
    undo_cursor: usize,

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
        orion_states: Arc<Mutex<OrionStates>>,
        orion_hid: Arc<Mutex<OrionHid>>,
        orion_paths: Arc<Mutex<OrionPaths>>,
        orion_persistence: Arc<Mutex<OrionPersistence>>,

        text_editor: Arc<Mutex<TextEditor>>,
    ) -> Self {
        let (file_dialog_tx, file_dialog_rx) = mpsc::channel();

        Self {
            orion_states,
            orion_hid,
            orion_paths,
            orion_persistence,

            file_dialog_tx,
            file_dialog_rx,

            text_editor,

            undo_buffer: Vec::new(),
            undo_cursor: 0,

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

    pub(crate) fn draw(&mut self, ui: &mut egui::Ui, orion_state: &mut OrionState) {
        orion_state.calculate_channel_counts();

        Self::draw_base_graphics(ui);

        self.draw_undo(ui);
        self.draw_save(ui, orion_state);

        self.draw_signals(ui, orion_state.signals);
        self.draw_text(ui, &orion_state.labels);

        self.draw_sources(ui, orion_state);
        self.draw_targets(ui, orion_state);
        self.draw_drag(ui, orion_state);

        self.clear_target_selection_when_outside(ui);
        self.update_targets_on_mouseup(ui, orion_state);
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

        if undo_response.is_pointer_button_down_on() != undo_response.contains_pointer() {
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

        if redo_response.is_pointer_button_down_on() != redo_response.contains_pointer() {
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

    fn draw_save(&mut self, ui: &mut egui::Ui, orion_state: &OrionState) {
        let save_response = render_asset!(
            ui,
            ControlPanelRoutingDimensions::NAV_X,
            ControlPanelRoutingDimensions::NAV_SAVE_Y,
            "SaveRoutingButton/default/released",
            1.0,
            0.0,
            Some(egui::Sense::click())
        );

        if save_response.is_pointer_button_down_on() != save_response.contains_pointer() {
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

        if load_response.is_pointer_button_down_on() != load_response.contains_pointer() {
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
            let dialog = self.orion_paths.lock().unwrap().create_file_dialog();
            let orion_state = orion_state.clone();

            std::thread::spawn(move || {
                if let Some(file) = dialog.save_file() {
                    let orion_file = OrionFile::new(file);

                    orion_file.encode(&orion_state);
                }
            });
        }

        if load_response.clicked() {
            let dialog = self.orion_paths.lock().unwrap().create_file_dialog();
            let tx = self.file_dialog_tx.clone();
            let ctx = ui.ctx().clone();

            std::thread::spawn(move || {
                let result = dialog.pick_file().map(|file| {
                    let orion_file = OrionFile::new(file);
                    orion_file.decode()
                });

                let _ = tx.send(result);

                ctx.request_repaint();
            });
        }

        while let Ok(result) = self.file_dialog_rx.try_recv() {
            if let Some((labels, routes)) = result {
                if let Some(orion_state) = self.orion_states.lock().unwrap().current_mut() {
                    orion_state.labels = std::sync::Arc::new(labels);

                    self.orion_persistence.lock().unwrap().write(orion_state, 0);
                }

                {
                    let mut orion_hid = self.orion_hid.lock().unwrap();

                    for (row, row_routes) in routes.iter().enumerate() {
                        orion_hid.queue_routes(row as u8, row_routes);
                    }
                }

                self.clear_undo();
            }
        }
    }

    fn draw_source(&mut self, ui: &mut egui::Ui, column: u8, row: u8, orion_state: &OrionState) {
        let label = &orion_state.labels[row as usize][column as usize];
        let (pos_x, pos_y) = Self::cell_pos(column, row, true);

        let color = if self.source_select_row == row && self.source_select_columns.contains(&column)
        {
            RoutingLabelColors::SELECT
        } else {
            orion_state.color_for_route(row, true)
        };

        let rect = egui::Rect::from_min_size(
            egui::pos2(pos_x, pos_y),
            egui::vec2(
                ControlPanelRoutingDimensions::SIZE_X,
                ControlPanelRoutingDimensions::SIZE_Y,
            ),
        );

        ui.painter().rect_filled(rect, 0.0, color);

        let field_id = egui::Id::new(("route", column, row));
        let id = ui.id().with(field_id);
        let response = ui.interact(Self::hitbox_rect(rect), id, egui::Sense::click());

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
            field_id,
            label,
            WidgetDimensions::TEXT_SIZE,
            WidgetColors::TEXT_COLOR_LABEL,
        );
    }

    fn draw_source_editing(
        &mut self,
        ui: &mut egui::Ui,
        column: u8,
        row: u8,
        orion_state: &OrionState,
    ) {
        let label = &orion_state.labels[row as usize][column as usize];
        let (pos_x, pos_y) = Self::cell_pos(column, row, true);
        let color = orion_state.color_for_route(row, true);

        let rect = egui::Rect::from_min_size(
            egui::pos2(
                pos_x + ControlPanelRoutingDimensions::EDIT_SIZE_OFFSET_X,
                pos_y + ControlPanelRoutingDimensions::EDIT_SIZE_OFFSET_Y,
            ),
            egui::vec2(
                ControlPanelRoutingDimensions::EDIT_SIZE_X,
                ControlPanelRoutingDimensions::EDIT_SIZE_Y,
            ),
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
            egui::Id::new(("route", column, row)),
            label,
            WidgetDimensions::TEXT_SIZE,
            WidgetColors::TEXT_COLOR_LABEL,
        );

        if let Some((route_text, tab)) = level_edit_result {
            let route_text = if route_text.trim().is_empty() {
                RoutingLabels::DEFAULT_LABEL_NUMBERS[row as usize][column as usize].to_string()
            } else {
                route_text
            };

            if let Some(orion_state) = self.orion_states.lock().unwrap().current_mut() {
                std::sync::Arc::make_mut(&mut orion_state.labels)[row as usize][column as usize] =
                    route_text;

                let preset = orion_state.preset_standby & 0x07;

                self.orion_persistence
                    .lock()
                    .unwrap()
                    .write(orion_state, preset);
            }

            if let Some(tab) = tab {
                let mut column_next = column as i16 + tab as i16;
                let mut row_next = row as usize;
                let row_count = orion_state.source_counts.len() as i16;

                while column_next < 0
                    || column_next >= orion_state.source_counts[row_next] as i16
                    || orion_state.source_counts[row_next] == 0
                {
                    row_next = (row_next as i16 + tab as i16).rem_euclid(row_count) as usize;

                    column_next = if tab == 1 {
                        0
                    } else {
                        orion_state.source_counts[row_next] as i16 - 1
                    };
                }

                self.label_edit_cell = Some([column_next as u8, row_next as u8]);

                self.text_editor
                    .lock()
                    .unwrap()
                    .set_editing_id(egui::Id::new(("route", column_next as u8, row_next as u8)));

                self.source_select_row = row_next as u8;
                self.source_select_columns.clear();
                self.source_select_columns.insert(column_next as u8);
            } else {
                self.label_edit_cell = None;
            }
        }

        if self.label_edit_cell.is_some() && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.label_edit_cell = None;
        }
    }

    fn draw_sources(&mut self, ui: &mut egui::Ui, orion_state: &OrionState) {
        for row in 0..orion_state.source_counts.len() as u8 {
            for column in 0..orion_state.source_counts[row as usize] as u8 {
                if Some([column, row]) != self.label_edit_cell {
                    self.draw_source(ui, column, row, orion_state);
                }
            }
        }

        if let Some([column, row]) = self.label_edit_cell {
            self.draw_source_editing(ui, column, row, orion_state);
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

        for row in 0..orion_state.target_counts.len() {
            for column in 0..orion_state.target_counts[row] {
                let routes = &orion_state.routes[row];
                let route = routes[column];

                let color = if self.target_select_cell == Some([column as u8, row as u8]) {
                    RoutingLabelColors::SELECT
                } else {
                    orion_state.color_for_route(route, true)
                };

                let label = orion_state.label_for_route(route, true);

                let rect =
                    Self::draw_rect_with_label(ui, column as u8, row as u8, false, color, label);

                let id = ui.id().with(("route_target", column, row));
                let response = ui.interact(Self::hitbox_rect(rect), id, egui::Sense::click());

                if response.contains_pointer() {
                    self.target_hover_cell = Some([column as u8, row as u8]);

                    if ControlPanelHelpers::mouse_down_on(&response) {
                        self.update_target_selections(column as u8, row as u8, route, orion_state);
                    }
                }

                egui::Popup::context_menu(&response)
                    .frame(egui::Frame::popup(ui.style()).fill(WidgetColors::BG_COLOR))
                    .show(|ui| {
                        if ui
                            .button(
                                egui::RichText::new("Mute")
                                    .size(WidgetDimensions::TEXT_SIZE)
                                    .color(WidgetColors::TEXT_COLOR),
                            )
                            .clicked()
                        {
                            self.update_targets_from_context_menu(
                                routes,
                                Some(column),
                                row,
                                Self::route_for_special(RoutingColumns::MUTE),
                                orion_state,
                            );

                            ui.close();
                        }

                        if ui
                            .button(
                                egui::RichText::new("Mute Row")
                                    .size(WidgetDimensions::TEXT_SIZE)
                                    .color(WidgetColors::TEXT_COLOR),
                            )
                            .clicked()
                        {
                            self.update_targets_from_context_menu(
                                routes,
                                None,
                                row,
                                Self::route_for_special(RoutingColumns::MUTE),
                                orion_state,
                            );

                            ui.close();
                        }

                        if ui
                            .button(
                                egui::RichText::new("Oscillator1")
                                    .size(WidgetDimensions::TEXT_SIZE)
                                    .color(WidgetColors::TEXT_COLOR),
                            )
                            .clicked()
                        {
                            self.update_targets_from_context_menu(
                                routes,
                                Some(column),
                                row,
                                Self::route_for_special(RoutingColumns::OSC_1),
                                orion_state,
                            );

                            ui.close();
                        }

                        if ui
                            .button(
                                egui::RichText::new("Oscillator2")
                                    .size(WidgetDimensions::TEXT_SIZE)
                                    .color(WidgetColors::TEXT_COLOR),
                            )
                            .clicked()
                        {
                            self.update_targets_from_context_menu(
                                routes,
                                Some(column),
                                row,
                                Self::route_for_special(RoutingColumns::OSC_2),
                                orion_state,
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
        orion_state: &OrionState,
    ) {
        let mut new_routes = *old_routes;

        if let Some(column) = column {
            new_routes[column] = route;
        } else {
            new_routes[..orion_state.target_counts[row]].fill(route);
        }

        self.queue_routes_with_undo(row as u8, old_routes, &new_routes);
    }

    fn update_target_selections(
        &mut self,
        column: u8,
        row: u8,
        route: u8,
        orion_state: &OrionState,
    ) {
        self.target_select_cell = Some([column, row]);

        self.source_select_toggle_columns.clear();
        self.source_select_columns.clear();

        if let Some([source_column, source_row]) = self.source_cell_for_route(route, orion_state) {
            self.source_select_row = source_row;
            self.source_select_columns.insert(source_column);
        }
    }

    fn draw_text(&self, ui: &mut egui::Ui, labels: &[[String; 32]; 7]) {
        if self.source_select_columns.is_empty() {
            return;
        }

        let mut text = String::new();

        text.push_str(Self::SOURCE_ROW_LABELS[self.source_select_row as usize]);
        text.push_str(": ");

        for (index, &column) in self.source_select_columns.iter().enumerate() {
            if index != 0 {
                text.push_str(", ");
            }

            text.push_str(&labels[self.source_select_row as usize][column as usize]);
        }

        ui.painter().text(
            egui::pos2(
                ControlPanelRoutingDimensions::TEXT_X,
                ControlPanelRoutingDimensions::TEXT_Y,
            ),
            egui::Align2::LEFT_TOP,
            text,
            egui::FontId::proportional(WidgetDimensions::TEXT_SIZE),
            WidgetColors::TEXT_COLOR_LABEL,
        );
    }

    fn draw_signals(&self, ui: &mut egui::Ui, signals: u8) {
        if signals & OrionStateSignals::USB != 0 {
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

        if signals & OrionStateSignals::MADI != 0 {
            render_asset!(
                ui,
                ControlPanelRoutingDimensions::PRESENSE_X,
                ControlPanelRoutingDimensions::PRESENSE_MADI_IN_Y,
                "PresenceLed/default/pressed",
                1.0,
                0.0,
                None
            );
        } else {
            render_asset!(
                ui,
                ControlPanelRoutingDimensions::PRESENSE_X,
                ControlPanelRoutingDimensions::PRESENSE_MADI_IN_Y,
                "PresenceLed/default/released",
                1.0,
                0.0,
                None
            );
        }

        if signals & OrionStateSignals::ADAT_1 != 0 {
            render_asset!(
                ui,
                ControlPanelRoutingDimensions::PRESENSE_X,
                ControlPanelRoutingDimensions::PRESENSE_ADAT_IN_Y,
                "PresenceLed/default/pressed",
                1.0,
                0.0,
                None
            );
        } else {
            render_asset!(
                ui,
                ControlPanelRoutingDimensions::PRESENSE_X,
                ControlPanelRoutingDimensions::PRESENSE_ADAT_IN_Y,
                "PresenceLed/default/released",
                1.0,
                0.0,
                None
            );
        }

        if signals & OrionStateSignals::ADAT_2 != 0 {
            render_asset!(
                ui,
                ControlPanelRoutingDimensions::PRESENSE_ADAT_IN_2_X,
                ControlPanelRoutingDimensions::PRESENSE_ADAT_IN_Y,
                "PresenceLed/default/pressed",
                1.0,
                0.0,
                None
            );
        } else {
            render_asset!(
                ui,
                ControlPanelRoutingDimensions::PRESENSE_ADAT_IN_2_X,
                ControlPanelRoutingDimensions::PRESENSE_ADAT_IN_Y,
                "PresenceLed/default/released",
                1.0,
                0.0,
                None
            );
        }

        if signals & OrionStateSignals::SPDIF != 0 {
            render_asset!(
                ui,
                ControlPanelRoutingDimensions::PRESENSE_X,
                ControlPanelRoutingDimensions::PRESENSE_SPDIF_IN_Y,
                "PresenceLed/default/pressed",
                1.0,
                0.0,
                None
            );
        } else {
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
    }

    fn clear_target_selection_when_outside(&mut self, ui: &mut egui::Ui) {
        if self.target_select_cell.is_none() {
            return;
        }

        let rect = egui::Rect::from_min_max(
            egui::pos2(
                ControlPanelRoutingDimensions::TARGET_ROUTES_MIN_X,
                ControlPanelRoutingDimensions::TARGET_ROUTES_MIN_Y,
            ),
            egui::pos2(
                ControlPanelRoutingDimensions::TARGET_ROUTES_MAX_X,
                ControlPanelRoutingDimensions::TARGET_ROUTES_MAX_Y,
            ),
        );

        if !ui.rect_contains_pointer(rect) {
            self.clear_selections();
        }
    }

    fn update_targets_on_mouseup(&mut self, ui: &mut egui::Ui, orion_state: &OrionState) {
        if self.source_mousedown && ui.input(|i| i.pointer.primary_released()) {
            if let Some([target_column, target_row]) = self.target_hover_cell {
                let old_routes = orion_state.routes[target_row as usize];
                let mut new_routes = old_routes;

                let target_row_count = orion_state.target_counts[target_row as usize];

                for (column_index, &source_column) in self.source_select_columns.iter().enumerate()
                {
                    if target_column as usize + column_index >= target_row_count {
                        break;
                    }

                    let route = source_column * RoutingColumns::STEP + self.source_select_row;

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
            let color = orion_state.color_for_route(self.source_select_row, true);

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

    fn source_cell_for_route(&self, route: u8, orion_state: &OrionState) -> Option<[u8; 2]> {
        let row = route % RoutingColumns::STEP;
        let column = route / RoutingColumns::STEP;

        if row == RoutingRows::SPECIAL {
            return None;
        }

        if row == RoutingRows::USB && column as usize >= orion_state.source_counts[1] {
            return None;
        }

        Some([column, row])
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

    fn hitbox_rect(rect: egui::Rect) -> egui::Rect {
        egui::Rect::from_min_size(
            rect.min,
            egui::vec2(
                ControlPanelRoutingDimensions::HITBOX_SIZE_X,
                ControlPanelRoutingDimensions::HITBOX_SIZE_Y,
            ),
        )
    }

    fn draw_rect_with_label(
        ui: &mut egui::Ui,
        column: u8,
        row: u8,
        is_source: bool,
        color: egui::Color32,
        label: &str,
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

        ui.painter().with_clip_rect(rect).text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            label,
            egui::FontId::proportional(WidgetDimensions::TEXT_SIZE),
            WidgetColors::TEXT_COLOR_LABEL,
        );

        rect
    }

    fn route_for_special(special_column: u8) -> u8 {
        RoutingRows::SPECIAL | (special_column * RoutingColumns::STEP)
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
